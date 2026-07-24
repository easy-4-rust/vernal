//! Warp 严格 AOP 错误恢复 Service 对象。

use std::{
    convert::Infallible,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use tower::Service;
use vernal_tower::AopServiceError;
use warp::{http::Request, reply::Response};

use crate::WarpAopError;

/// 把严格 AOP 错误恢复成 Warp 原生 Response 的 Tower Service。
///
/// 该对象位于请求 Scope Service 内侧，所以策略短路产生的响应仍会经过统一 Body
/// 生命周期包装。`poll_ready` 阶段的异常会暂存到下一次 `call`，避免把可映射的
/// HTTP 失败错误地升级为连接级 Service 错误。
pub struct VernalWarpAopService<S> {
    inner: S,
    readiness_error: Option<AopServiceError<Infallible>>,
}

impl<S> VernalWarpAopService<S> {
    /// 包装已经织入严格 AOP 的 Warp Service。
    pub(crate) const fn new(inner: S) -> Self {
        Self {
            inner,
            readiness_error: None,
        }
    }
}

impl<S> Clone for VernalWarpAopService<S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        // Readiness 错误属于当前 Service 实例的下一次调用，不能复制到克隆实例。
        Self::new(self.inner.clone())
    }
}

impl<S, B> Service<Request<B>> for VernalWarpAopService<S>
where
    S: Service<Request<B>, Response = Response, Error = AopServiceError<Infallible>>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = Response;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, context: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.inner.poll_ready(context) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(error)) => {
                self.readiness_error = Some(error);
                Poll::Ready(Ok(()))
            }
        }
    }

    fn call(&mut self, request: Request<B>) -> Self::Future {
        if let Some(error) = self.readiness_error.take() {
            return Box::pin(async move { Ok(WarpAopError::new(error).response()) });
        }

        // Tower 要求调用刚刚通过 readiness 检查的实例；以克隆替换字段后，将
        // 原实例和 owned Request 一起移动进 Future。
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        Box::pin(async move {
            match inner.call(request).await {
                Ok(response) => Ok(response),
                Err(error) => Ok(WarpAopError::new(error).response()),
            }
        })
    }
}
