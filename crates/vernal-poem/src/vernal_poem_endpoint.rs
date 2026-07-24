//! Poem 请求生命周期 Endpoint 对象。

use std::sync::Arc;

use poem::{Body, Endpoint, IntoResponse, Request, Response, Result};
use tokio_util::sync::CancellationToken;
use vernal_context::ApplicationContext;
use vernal_web::WebRequestScope;

use crate::{PoemRejection, PoemScopedStream};

/// 执行 Poem 请求 Context 注入、Scope 创建和响应 Body 包装。
pub struct VernalPoemEndpoint<E> {
    endpoint: E,
    context: Arc<ApplicationContext>,
}

impl<E> VernalPoemEndpoint<E> {
    /// 由中间件工厂创建请求 Endpoint。
    pub(crate) fn new(endpoint: E, context: Arc<ApplicationContext>) -> Self {
        Self { endpoint, context }
    }
}

impl<E> Endpoint for VernalPoemEndpoint<E>
where
    E: Endpoint,
{
    type Output = Response;

    async fn call(&self, mut request: Request) -> Result<Self::Output> {
        request.extensions_mut().insert(Arc::clone(&self.context));
        let cancellation = CancellationToken::new();
        let scope = Arc::new(WebRequestScope::new(cancellation.clone()));
        request.extensions_mut().insert(Arc::clone(&scope));

        // 请求 Future 或响应流被丢弃时，由 DropGuard 只发出同步取消信号；
        // 这里预先启动的 Tokio 任务负责执行真正的异步 Scope 关闭。
        let cleanup_scope = Arc::clone(&scope);
        let cleanup_cancellation = cancellation.clone();
        tokio::spawn(async move {
            cleanup_cancellation.cancelled().await;
            let _ = cleanup_scope.close().await;
        });

        let request_guard = cancellation.drop_guard();
        match self.endpoint.call(request).await {
            Ok(output) => {
                let cancellation = request_guard.disarm();
                let mut response = output.into_response();
                let body_stream = response.take_body().into_bytes_stream();
                response.set_body(Body::from_bytes_stream(PoemScopedStream::new(
                    body_stream,
                    scope,
                    cancellation,
                )));
                Ok(response)
            }
            Err(error) => {
                request_guard.disarm();
                scope.close().await.map_err(PoemRejection::scope_close)?;
                Err(error)
            }
        }
    }
}
