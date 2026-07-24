//! Gotham 请求生命周期 Middleware 对象。

use std::{panic::AssertUnwindSafe, pin::Pin, sync::Arc};

use gotham::{
    handler::{HandlerError, HandlerFuture},
    middleware::{Middleware, NewMiddleware},
    state::State,
};
use http::Response;
use http_body_util::BodyExt;
use tokio_util::sync::CancellationToken;
use vernal_context::ApplicationContext;
use vernal_web::WebRequestScope;

use crate::{GothamRejection, GothamScopedBody, VernalGothamContext, VernalGothamRequestScope};

/// 为 Gotham Pipeline 注入 Context，并让请求 Scope 跟随响应 Body。
pub struct VernalGothamMiddleware {
    // Gotham 只在中间件工厂引用跨越 panic 捕获边界时要求 RefUnwindSafe。
    // Context 内部状态仍由 Tokio 锁保护；这里不绕过任何线程安全约束。
    context: AssertUnwindSafe<Arc<ApplicationContext>>,
}

impl Clone for VernalGothamMiddleware {
    fn clone(&self) -> Self {
        Self::new(Arc::clone(&self.context.0))
    }
}

impl VernalGothamMiddleware {
    /// 创建 Gotham 中间件。
    #[must_use]
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self {
            context: AssertUnwindSafe(context),
        }
    }
}

impl NewMiddleware for VernalGothamMiddleware {
    type Instance = Self;

    fn new_middleware(&self) -> gotham::anyhow::Result<Self::Instance> {
        Ok(self.clone())
    }
}

impl Middleware for VernalGothamMiddleware {
    fn call<Chain>(self, mut state: State, chain: Chain) -> Pin<Box<HandlerFuture>>
    where
        Chain: FnOnce(State) -> Pin<Box<HandlerFuture>> + Send + 'static,
    {
        state.put(VernalGothamContext(Arc::clone(&self.context.0)));
        let cancellation = CancellationToken::new();
        let scope = Arc::new(WebRequestScope::new(cancellation.clone()));
        state.put(VernalGothamRequestScope(Arc::clone(&scope)));

        // Handler Future 或响应 Body 被丢弃时，DropGuard 发出同步取消信号；
        // Tokio 清理任务负责执行异步关闭钩子。
        let cleanup_scope = Arc::clone(&scope);
        let cleanup_cancellation = cancellation.clone();
        tokio::spawn(async move {
            cleanup_cancellation.cancelled().await;
            let _ = cleanup_scope.close().await;
        });

        let request_guard = cancellation.drop_guard();
        Box::pin(async move {
            match chain(state).await {
                Ok((state, response)) => {
                    let cancellation = request_guard.disarm();
                    let (parts, body) = response.into_parts();
                    let body = GothamScopedBody::new(body, scope, cancellation).boxed_unsync();
                    Ok((state, Response::from_parts(parts, body)))
                }
                Err((state, error)) => {
                    request_guard.disarm();
                    match scope.close().await {
                        Ok(()) => Err((state, error)),
                        Err(error) => Err((
                            state,
                            HandlerError::from(GothamRejection::scope_close(error)),
                        )),
                    }
                }
            }
        })
    }
}
