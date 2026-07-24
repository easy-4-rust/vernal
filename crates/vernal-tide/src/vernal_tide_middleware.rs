//! Tide 请求生命周期与严格 AOP Middleware 对象。

use std::sync::Arc;

use futures_lite::io::BufReader;
use tide::{Body, Middleware, Next as TideNext, Request, Response};
use tokio_util::sync::CancellationToken;
use vernal_aop::{BorrowedInvocationTarget, InvocationError};
use vernal_context::ApplicationContext;
use vernal_web::{HandlerInvocation, RequestContext, RouteMetadata, WebRequestScope};

use crate::{
    TideAopError, TideResponse, TideScopedReader, tide_borrowed_target::TideBorrowedTarget,
    tide_request_snapshot::TideRequestSnapshot,
};

/// 为 Tide 请求注入 Context，并让请求 Scope 跟随响应 Body。
#[derive(Clone)]
pub struct VernalTideMiddleware {
    context: Arc<ApplicationContext>,
    strict_aop_path_pattern: Option<Arc<str>>,
}

impl VernalTideMiddleware {
    /// 创建 Tide 中间件。
    #[must_use]
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self {
            context,
            strict_aop_path_pattern: None,
        }
    }

    /// 创建同时管理请求 Scope 并执行严格 Send-AOP 的 Middleware。
    ///
    /// Tide 公共 Request 只暴露匹配参数值，不暴露 Router 的原始路径模板。调用方
    /// 应把本中间件安装到具体 `Route`，并传入同一条完整低基数模式。空模式或
    /// 缺少预编译计划时 fail-closed，不回退到含用户输入的 URL。
    #[must_use]
    pub fn strict_aop(context: Arc<ApplicationContext>, path_pattern: impl Into<Arc<str>>) -> Self {
        Self {
            context,
            strict_aop_path_pattern: Some(path_pattern.into()),
        }
    }

    /// 通过借用型 Send-AOP 目标驱动完整 Tide Middleware/Endpoint 链。
    async fn call_with_aop<State>(
        &self,
        mut request: Request<State>,
        next: TideNext<'_, State>,
        scope: Arc<WebRequestScope>,
        cancellation: CancellationToken,
        path_pattern: Arc<str>,
    ) -> Result<Response, TideAopError>
    where
        State: Clone + Send + Sync + 'static,
    {
        if path_pattern.is_empty() {
            return Err(TideAopError::MissingRouteMetadata);
        }
        let method = request.method().to_string();
        let route =
            RouteMetadata::new(Arc::clone(&path_pattern), method, Arc::clone(&path_pattern));
        let operation = route.aop_operation();
        let snapshot = TideRequestSnapshot::capture(&request)?;
        let request_context = Arc::new(RequestContext::new(route, cancellation));
        request_context.extensions().insert(snapshot).await;
        request.set_ext(Arc::clone(&request_context));
        let invocation = HandlerInvocation::new(request_context, scope)
            .aop_invocation()
            .await;
        let plan = self
            .context
            .invocation_plans()
            .get(&operation)
            .cloned()
            .ok_or_else(|| {
                TideAopError::invocation(InvocationError::PlanNotFound {
                    operation: operation.clone(),
                })
            })?;

        let envelope = Arc::new(TideResponse::empty());
        let result = {
            let mut target = TideBorrowedTarget::new(request, next, Arc::clone(&envelope));
            plan.invoke_borrowed(invocation, &mut target as &mut dyn BorrowedInvocationTarget)
                .await
        };
        let value = result.map_err(TideAopError::invocation)?;
        let returned = value
            .downcast::<Arc<TideResponse>>()
            .map_err(|_| TideAopError::ResponseTypeMismatch)?;
        returned
            .take()
            .await
            .ok_or(TideAopError::ResponseUnavailable)
    }
}

#[tide::utils::async_trait]
impl<State> Middleware<State> for VernalTideMiddleware
where
    State: Clone + Send + Sync + 'static,
{
    async fn handle(&self, mut request: Request<State>, next: TideNext<'_, State>) -> tide::Result {
        request.set_ext(Arc::clone(&self.context));
        let scope = Arc::new(WebRequestScope::from_application_context(Arc::clone(
            &self.context,
        )));
        let cancellation = scope.cancellation().clone();
        request.set_ext(Arc::clone(&scope));

        // 请求 Future 或 Body 被丢弃时，DropGuard 发出同步取消信号；
        // Vernal 的 Tokio runtime 负责执行真正的异步关闭钩子。
        let cleanup_scope = Arc::clone(&scope);
        let cleanup_cancellation = cancellation.clone();
        tokio::spawn(async move {
            cleanup_cancellation.cancelled().await;
            let _ = cleanup_scope.close().await;
        });

        let request_guard = cancellation.clone().drop_guard();
        let mut response = match &self.strict_aop_path_pattern {
            Some(path_pattern) => self
                .call_with_aop(
                    request,
                    next,
                    Arc::clone(&scope),
                    cancellation,
                    Arc::clone(path_pattern),
                )
                .await
                .unwrap_or_else(|error| error.response()),
            None => next.run(request).await,
        };
        let cancellation = request_guard.disarm();
        let length = response.len();
        let body = response.take_body();
        let reader = BufReader::new(TideScopedReader::new(body, scope, cancellation));
        response.set_body(Body::from_reader(reader, length));
        Ok(response)
    }
}
