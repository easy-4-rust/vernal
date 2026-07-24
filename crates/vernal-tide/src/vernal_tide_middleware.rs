//! Tide 请求生命周期 Middleware 对象。

use std::sync::Arc;

use futures_lite::io::BufReader;
use tide::{Body, Middleware, Next, Request};
use tokio_util::sync::CancellationToken;
use vernal_context::ApplicationContext;
use vernal_web::WebRequestScope;

use crate::TideScopedReader;

/// 为 Tide 请求注入 Context，并让请求 Scope 跟随响应 Body。
#[derive(Clone)]
pub struct VernalTideMiddleware {
    context: Arc<ApplicationContext>,
}

impl VernalTideMiddleware {
    /// 创建 Tide 中间件。
    #[must_use]
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self { context }
    }
}

#[tide::utils::async_trait]
impl<State> Middleware<State> for VernalTideMiddleware
where
    State: Clone + Send + Sync + 'static,
{
    async fn handle(&self, mut request: Request<State>, next: Next<'_, State>) -> tide::Result {
        request.set_ext(Arc::clone(&self.context));
        let cancellation = CancellationToken::new();
        let scope = Arc::new(WebRequestScope::new(cancellation.clone()));
        request.set_ext(Arc::clone(&scope));

        // 请求 Future 或 Body 被丢弃时，DropGuard 发出同步取消信号；
        // Vernal 的 Tokio runtime 负责执行真正的异步关闭钩子。
        let cleanup_scope = Arc::clone(&scope);
        let cleanup_cancellation = cancellation.clone();
        tokio::spawn(async move {
            cleanup_cancellation.cancelled().await;
            let _ = cleanup_scope.close().await;
        });

        let request_guard = cancellation.drop_guard();
        let mut response = next.run(request).await;
        let cancellation = request_guard.disarm();
        let length = response.len();
        let body = response.take_body();
        let reader = BufReader::new(TideScopedReader::new(body, scope, cancellation));
        response.set_body(Body::from_reader(reader, length));
        Ok(response)
    }
}
