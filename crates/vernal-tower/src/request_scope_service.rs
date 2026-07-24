//! 请求作用域 Service 对象。

use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use http::{Request, Response};
use tokio_util::sync::CancellationToken;
use tower::Service;
use vernal_web::WebRequestScope;

use crate::{ScopedBody, TowerError};

/// 在请求 Future 与响应 Body 的完整生命周期内管理 `WebRequestScope`。
#[derive(Clone)]
pub struct RequestScopeService<S> {
    inner: S,
}

impl<S> RequestScopeService<S> {
    /// 包装下游 Service。
    pub(crate) fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S, RequestBody, ResponseBody> Service<Request<RequestBody>> for RequestScopeService<S>
where
    S: Service<Request<RequestBody>, Response = Response<ResponseBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
    RequestBody: Send + 'static,
    ResponseBody: http_body::Body + Send + 'static,
{
    type Response = Response<ScopedBody<ResponseBody>>;
    type Error = TowerError<S::Error>;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, context: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(context).map_err(TowerError::Upstream)
    }

    fn call(&mut self, mut request: Request<RequestBody>) -> Self::Future {
        let cancellation = CancellationToken::new();
        let scope = Arc::new(WebRequestScope::new(cancellation.clone()));
        request.extensions_mut().insert(Arc::clone(&scope));

        // 清理 task 在请求开始时建立；Future/Body Drop 只发取消信号。
        let cleanup_scope = Arc::clone(&scope);
        let cleanup_cancellation = cancellation.clone();
        tokio::spawn(async move {
            cleanup_cancellation.cancelled().await;
            let _ = cleanup_scope.close().await;
        });

        let request_guard = cancellation.clone().drop_guard();
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        Box::pin(async move {
            match inner.call(request).await {
                Ok(response) => {
                    let cancellation = request_guard.disarm();
                    let (parts, body) = response.into_parts();
                    Ok(Response::from_parts(
                        parts,
                        ScopedBody::new(body, scope, cancellation),
                    ))
                }
                Err(error) => {
                    // 显式关闭路径先持有 Scope 操作锁，再由 Scope 发出取消。
                    // 这样后台兜底任务不会抢先执行并吞掉关闭钩子错误。
                    request_guard.disarm();
                    scope.close().await.map_err(TowerError::Scope)?;
                    Err(TowerError::Upstream(error))
                }
            }
        })
    }
}
