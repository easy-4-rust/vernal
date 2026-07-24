//! Gotham 请求 Scope 感知响应 Body 对象。

use std::{
    future::Future,
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use bytes::Bytes;
use http_body::{Body, Frame, SizeHint};
use tokio_util::sync::{CancellationToken, DropGuard};
use vernal_web::{ScopeError, WebRequestScope};

use crate::GothamBodyError;

type CloseFuture = Pin<Box<dyn Future<Output = Result<(), ScopeError>> + Send + 'static>>;

/// 将 Gotham 的 `http-body` 响应生命周期与 `WebRequestScope` 绑定。
///
/// Gotham 0.8 使用 `http-body` 1.0，因此该包装器可以保留 Data Frame、Trailer、
/// Size Hint、上游错误与背压。Scope 只在 EOF 或错误后关闭；Body 被提前丢弃时，
/// [`DropGuard`] 发出取消信号，由中间件预先启动的 Tokio 任务兜底清理。
pub struct GothamScopedBody<B> {
    inner: Pin<Box<B>>,
    scope: Arc<WebRequestScope>,
    drop_guard: Option<DropGuard>,
    closing: Option<CloseFuture>,
    upstream_error: Option<io::Error>,
    completed: bool,
}

impl<B> GothamScopedBody<B> {
    /// 包装 Gotham 原生响应 Body。
    #[must_use]
    pub fn new(inner: B, scope: Arc<WebRequestScope>, cancellation: CancellationToken) -> Self {
        Self {
            inner: Box::pin(inner),
            scope,
            drop_guard: Some(cancellation.drop_guard()),
            closing: None,
            upstream_error: None,
            completed: false,
        }
    }

    /// 将同步 Body 轮询切换到异步 Scope 关闭阶段。
    fn begin_close(&mut self) {
        if self.closing.is_some() || self.completed {
            return;
        }
        if let Some(drop_guard) = self.drop_guard.take() {
            drop_guard.disarm();
        }
        let scope = Arc::clone(&self.scope);
        self.closing = Some(Box::pin(async move { scope.close().await }));
    }
}

impl<B> Body for GothamScopedBody<B>
where
    B: Body<Data = Bytes, Error = io::Error> + 'static,
{
    type Data = Bytes;
    type Error = io::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let body = self.as_mut().get_mut();
        if body.completed {
            return Poll::Ready(None);
        }

        if let Some(closing) = &mut body.closing {
            return match closing.as_mut().poll(context) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(error)) => {
                    body.closing = None;
                    body.completed = true;
                    Poll::Ready(Some(Err(io::Error::other(GothamBodyError::scope_close(
                        error,
                    )))))
                }
                Poll::Ready(Ok(())) => {
                    body.closing = None;
                    body.completed = true;
                    match body.upstream_error.take() {
                        Some(error) => Poll::Ready(Some(Err(error))),
                        None => Poll::Ready(None),
                    }
                }
            };
        }

        match body.inner.as_mut().poll_frame(context) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(Ok(frame))) => Poll::Ready(Some(Ok(frame))),
            Poll::Ready(Some(Err(error))) => {
                body.upstream_error = Some(error);
                body.begin_close();
                context.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Ready(None) => {
                body.begin_close();
                context.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }

    fn is_end_stream(&self) -> bool {
        self.completed || (self.closing.is_none() && self.inner.is_end_stream())
    }

    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}
