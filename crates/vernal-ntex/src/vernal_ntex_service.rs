//! Ntex 请求生命周期 Service 对象。

use std::{any::type_name, cell::RefCell, fmt, rc::Rc, sync::Arc};

use ntex::{
    http::body::{Body, ResponseBody},
    service::{Service, ServiceCtx},
    web::{ErrorRenderer, WebRequest, WebResponse},
};
use tokio_util::sync::CancellationToken;
use vernal_aop::{BorrowedLocalInvocationTarget, LocalInvocationError};
use vernal_context::ApplicationContext;
use vernal_web::{HandlerInvocation, RequestContext, RouteMetadata, WebRequestScope};

use crate::{
    NtexAopError, NtexRejection, NtexScopedBody, ntex_borrowed_target::NtexBorrowedTarget,
    ntex_request_snapshot::NtexRequestSnapshot, ntex_response_envelope::NtexResponseEnvelope,
    ntex_service_error::NtexServiceError,
};

/// 执行 Ntex 请求 Context 注入、Scope 创建和响应 Body 包装。
pub struct VernalNtexService<S> {
    service: S,
    context: Arc<ApplicationContext>,
    strict_aop_path_pattern: Option<Arc<str>>,
}

impl<S> VernalNtexService<S> {
    /// 由中间件工厂创建请求 Service。
    pub(crate) fn new(
        service: S,
        context: Arc<ApplicationContext>,
        strict_aop_path_pattern: Option<Arc<str>>,
    ) -> Self {
        Self {
            service,
            context,
            strict_aop_path_pattern,
        }
    }
}

impl<S> VernalNtexService<S> {
    /// 用仍可用的 Ntex Request 或 Response 所有权构造 AOP 失败响应。
    fn render_aop_error<Err, ServiceError>(
        request: &Rc<RefCell<Option<WebRequest<Err>>>>,
        response: &Rc<NtexResponseEnvelope>,
        error: NtexAopError,
    ) -> Result<WebResponse, ServiceError>
    where
        ServiceError: From<NtexRejection>,
        Err: ErrorRenderer,
    {
        if let Some(response) = response.take() {
            return Ok(error.replace_response(response));
        }
        if let Some(request) = request.borrow_mut().take() {
            return Ok(request.into_response(error.response()));
        }
        Err(NtexRejection::aop(error).into())
    }

    /// 使用显式 Resource Pattern 执行覆盖完整 Ntex Service Future 的本地环绕链。
    async fn call_with_aop<'a, Err>(
        &'a self,
        request: WebRequest<Err>,
        service_context: ServiceCtx<'a, Self>,
        scope: Arc<WebRequestScope>,
        cancellation: CancellationToken,
        path_pattern: Arc<str>,
    ) -> Result<WebResponse, S::Error>
    where
        S: Service<WebRequest<Err>, Response = WebResponse>,
        S::Error: From<NtexRejection> + fmt::Debug + fmt::Display + 'static,
        Err: ErrorRenderer,
    {
        if path_pattern.is_empty() {
            return Ok(request.into_response(NtexAopError::MissingResourcePattern.response()));
        }

        let method = request.method().as_str().to_owned();
        let route =
            RouteMetadata::new(Arc::clone(&path_pattern), method, Arc::clone(&path_pattern));
        let operation = route.aop_operation();
        let snapshot = NtexRequestSnapshot::capture(&request);
        let request_context = Arc::new(RequestContext::new(route, cancellation));
        request_context.extensions().insert(snapshot).await;
        request
            .extensions_mut()
            .insert(Arc::clone(&request_context));
        let invocation = HandlerInvocation::new(request_context, scope)
            .aop_invocation()
            .await;
        let Some(plan) = self
            .context
            .local_invocation_plans()
            .get(&operation)
            .cloned()
        else {
            return Ok(request.into_response(
                NtexAopError::invocation(LocalInvocationError::PlanNotFound {
                    operation: operation.clone(),
                })
                .response(),
            ));
        };

        // WebRequest 不可在调用下游前克隆，否则 Payload 的 Rc 唯一性会被破坏。
        // 请求槽与响应信封保证任意时刻只由一方拥有原生请求/响应。
        let request = Rc::new(RefCell::new(Some(request)));
        let response = Rc::new(NtexResponseEnvelope::new());
        let target: Rc<dyn BorrowedLocalInvocationTarget + 'a> = Rc::new(NtexBorrowedTarget::new(
            &self.service,
            service_context,
            Rc::clone(&request),
            Rc::clone(&response),
        ));

        match plan.invoke_borrowed(invocation, target).await {
            Ok(value) => {
                let Ok(returned) = value.downcast::<Rc<NtexResponseEnvelope>>() else {
                    return Self::render_aop_error(
                        &request,
                        &response,
                        NtexAopError::invocation(LocalInvocationError::ReturnTypeMismatch {
                            expected: type_name::<Rc<NtexResponseEnvelope>>(),
                        }),
                    );
                };
                if !Rc::ptr_eq(returned.as_ref(), &response) {
                    return Self::render_aop_error(
                        &request,
                        &response,
                        NtexAopError::ResponseEnvelopeMismatch,
                    );
                }
                response
                    .take()
                    .ok_or_else(|| NtexRejection::aop(NtexAopError::ResponseUnavailable).into())
            }
            Err(LocalInvocationError::Target { source }) => {
                // 原生 Ntex Service 错误穿过专用信封后原样返回；WebFailure 等策略
                // 错误才转换为稳定协议响应。
                match source.downcast::<NtexServiceError<S::Error>>() {
                    Ok(error) => Err(error.into_inner()),
                    Err(source) => Self::render_aop_error(
                        &request,
                        &response,
                        NtexAopError::invocation(LocalInvocationError::Target { source }),
                    ),
                }
            }
            Err(error) => {
                Self::render_aop_error(&request, &response, NtexAopError::invocation(error))
            }
        }
    }
}

impl<S, Err> Service<WebRequest<Err>> for VernalNtexService<S>
where
    S: Service<WebRequest<Err>, Response = WebResponse>,
    S::Error: From<NtexRejection> + fmt::Debug + fmt::Display + 'static,
    Err: ErrorRenderer,
{
    type Response = WebResponse;
    type Error = S::Error;

    async fn call(
        &self,
        request: WebRequest<Err>,
        service_context: ServiceCtx<'_, Self>,
    ) -> Result<Self::Response, Self::Error> {
        request.extensions_mut().insert(Arc::clone(&self.context));
        let scope = Arc::new(WebRequestScope::from_application_context(Arc::clone(
            &self.context,
        )));
        let cancellation = scope.cancellation().clone();
        request.extensions_mut().insert(Arc::clone(&scope));

        // Body 或请求 Future 被丢弃时，DropGuard 只能同步发出取消信号；
        // 预启动的本地任务负责执行真正的异步关闭钩子。
        let cleanup_scope = Arc::clone(&scope);
        let cleanup_cancellation = cancellation.clone();
        ntex::rt::spawn(async move {
            cleanup_cancellation.cancelled().await;
            let _ = cleanup_scope.close().await;
        });

        let request_guard = cancellation.clone().drop_guard();
        let response = match &self.strict_aop_path_pattern {
            Some(path_pattern) => {
                self.call_with_aop(
                    request,
                    service_context,
                    Arc::clone(&scope),
                    cancellation,
                    Arc::clone(path_pattern),
                )
                .await
            }
            None => service_context.call(&self.service, request).await,
        };
        match response {
            Ok(response) => {
                let cancellation = request_guard.disarm();
                Ok(response.map_body(move |_head, body| {
                    ResponseBody::Other(Body::from_message(NtexScopedBody::new(
                        body,
                        scope,
                        cancellation,
                    )))
                }))
            }
            Err(error) => {
                request_guard.disarm();
                scope.close().await.map_err(NtexRejection::scope_close)?;
                Err(error)
            }
        }
    }
}
