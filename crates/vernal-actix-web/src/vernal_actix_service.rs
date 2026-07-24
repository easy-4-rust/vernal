//! Actix Web 请求生命周期 Service 对象。

use std::{
    any::type_name,
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll},
};

use actix_web::{
    Error, HttpMessage,
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse},
};
use tokio_util::sync::CancellationToken;
use vernal_aop::{LocalInvocationError, LocalInvocationTarget, LocalInvocationValue};
use vernal_context::ApplicationContext;
use vernal_web::{HandlerInvocation, RequestContext, RouteMetadata, WebRequestScope};

use crate::{
    ActixAopError, ActixRejection, ActixScopedBody, actix_request_snapshot::ActixRequestSnapshot,
};

type ServiceFuture<B> =
    Pin<Box<dyn Future<Output = Result<ServiceResponse<ActixScopedBody<B>>, Error>>>>;

/// 执行 Actix 请求 Context 注入、Scope 创建和响应 Body 包装。
pub struct VernalActixService<S> {
    service: Rc<S>,
    context: Arc<ApplicationContext>,
    strict_aop: bool,
}

impl<S> VernalActixService<S> {
    /// 由中间件工厂创建请求 Service。
    pub(crate) fn new(service: Rc<S>, context: Arc<ApplicationContext>, strict_aop: bool) -> Self {
        Self {
            service,
            context,
            strict_aop,
        }
    }
}

impl<S> VernalActixService<S>
where
    S: 'static,
{
    /// 使用匹配后的 Actix Resource Pattern 执行完整本地环绕链。
    async fn call_with_aop<B>(
        service: Rc<S>,
        context: Arc<ApplicationContext>,
        mut request: ServiceRequest,
        scope: Arc<WebRequestScope>,
        cancellation: CancellationToken,
    ) -> Result<ServiceResponse<B>, Error>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: MessageBody + 'static,
    {
        let path_pattern = request
            .match_pattern()
            .ok_or(ActixAopError::MissingRouteMetadata)?;
        let method = request.method().as_str().to_owned();
        let route = RouteMetadata::new(path_pattern.clone(), method.clone(), path_pattern);
        let operation = route.aop_operation();
        let snapshot =
            ActixRequestSnapshot::capture(&request).map_err(ActixAopError::request_snapshot)?;
        let request_context = Arc::new(RequestContext::new(route, cancellation));
        request_context.extensions().insert(snapshot).await;
        request
            .extensions_mut()
            .insert(Arc::clone(&request_context));
        let invocation = HandlerInvocation::new(request_context, scope)
            .aop_invocation()
            .await;
        let plan = context
            .local_invocation_plans()
            .get(&operation)
            .cloned()
            .ok_or_else(|| {
                ActixAopError::invocation(LocalInvocationError::PlanNotFound {
                    operation: operation.clone(),
                })
            })?;

        // ServiceRequest 与下游 Future 都可能持有 Rc。一次性 RefCell 信封在
        // 调用目标前立即释放可变借用，不让 RefCell Guard 跨越 await。
        let request = Rc::new(RefCell::new(Some(request)));
        let target: Rc<LocalInvocationTarget> = Rc::new(move |_invocation| {
            let request = Rc::clone(&request);
            let service = Rc::clone(&service);
            Box::pin(async move {
                let request = request.borrow_mut().take().ok_or_else(|| {
                    LocalInvocationError::target(ActixAopError::RequestAlreadyTaken)
                })?;
                let response = service
                    .call(request)
                    .await
                    .map_err(LocalInvocationError::target)?;
                Ok(Box::new(response) as LocalInvocationValue)
            })
        });

        let value = match plan.invoke(invocation, target).await {
            Ok(value) => value,
            Err(LocalInvocationError::Target { source }) => {
                // 下游 Actix Error 保持原生 ResponseError 状态和正文；策略等非
                // Actix 错误才进入统一 Local-AOP 映射。
                match source.downcast::<Error>() {
                    Ok(error) => return Err(*error),
                    Err(source) => {
                        return Err(ActixAopError::invocation(LocalInvocationError::Target {
                            source,
                        })
                        .into());
                    }
                }
            }
            Err(error) => {
                return Err(ActixAopError::invocation(error).into());
            }
        };
        value
            .downcast::<ServiceResponse<B>>()
            .map(|response| *response)
            .map_err(|_| {
                ActixAopError::invocation(LocalInvocationError::ReturnTypeMismatch {
                    expected: type_name::<ServiceResponse<B>>(),
                })
                .into()
            })
    }
}

impl<S, B> Service<ServiceRequest> for VernalActixService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<ActixScopedBody<B>>;
    type Error = Error;
    type Future = ServiceFuture<B>;

    fn poll_ready(&self, context: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(context)
    }

    fn call(&self, request: ServiceRequest) -> Self::Future {
        request.extensions_mut().insert(Arc::clone(&self.context));
        let cancellation = CancellationToken::new();
        let scope = Arc::new(WebRequestScope::new(cancellation.clone()));
        request.extensions_mut().insert(Arc::clone(&scope));

        // 请求 Future 或响应 Body 被丢弃时，由 DropGuard 只发出同步取消信号，
        // 这里预先启动的任务负责执行真正的异步关闭。
        let cleanup_scope = Arc::clone(&scope);
        let cleanup_cancellation = cancellation.clone();
        actix_web::rt::spawn(async move {
            cleanup_cancellation.cancelled().await;
            let _ = cleanup_scope.close().await;
        });

        let request_guard = cancellation.clone().drop_guard();
        let service = Rc::clone(&self.service);
        let context = Arc::clone(&self.context);
        let strict_aop = self.strict_aop;
        Box::pin(async move {
            let response = if strict_aop {
                Self::call_with_aop(service, context, request, Arc::clone(&scope), cancellation)
                    .await
            } else {
                service.call(request).await
            };
            match response {
                Ok(response) => {
                    let cancellation = request_guard.disarm();
                    Ok(
                        response
                            .map_body(|_, body| ActixScopedBody::new(body, scope, cancellation)),
                    )
                }
                Err(error) => {
                    request_guard.disarm();
                    scope.close().await.map_err(ActixRejection::scope_close)?;
                    Err(error)
                }
            }
        })
    }
}
