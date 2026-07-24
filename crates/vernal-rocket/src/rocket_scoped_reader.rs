//! Rocket 请求 Scope 感知响应 Reader 对象。

use std::{
    future::Future,
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use rocket::{
    response::Body,
    tokio::io::{AsyncRead, ReadBuf},
};
use tokio_util::sync::{CancellationToken, DropGuard};
use vernal_web::{ScopeError, WebRequestScope};

type CloseFuture = Pin<Box<dyn Future<Output = Result<(), ScopeError>> + Send + 'static>>;

/// 将 Rocket 原生 `Body` 的异步读取生命周期与 `WebRequestScope` 绑定。
///
/// Rocket 0.5 的公开 Body 只暴露 `AsyncRead`，因此 Fairing 会把原响应转换为
/// streamed body。字节流、背压与错误保持原样，但已知长度分类无法通过公开 API
/// 原样重建；该边界在架构文档中显式记录。
pub struct RocketScopedReader<'r> {
    inner: Pin<Box<Body<'r>>>,
    scope: Arc<WebRequestScope>,
    drop_guard: Option<DropGuard>,
    closing: Option<CloseFuture>,
    upstream_error: Option<io::Error>,
    completed: bool,
}

impl<'r> RocketScopedReader<'r> {
    /// 包装 Rocket 原生响应 Body。
    #[must_use]
    pub fn new(
        inner: Body<'r>,
        scope: Arc<WebRequestScope>,
        cancellation: CancellationToken,
    ) -> Self {
        Self {
            inner: Box::pin(inner),
            scope,
            drop_guard: Some(cancellation.drop_guard()),
            closing: None,
            upstream_error: None,
            completed: false,
        }
    }

    /// 启动显式 Scope 关闭 Future。
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

impl AsyncRead for RocketScopedReader<'_> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let reader = self.as_mut().get_mut();
        if reader.completed {
            return Poll::Ready(Ok(()));
        }

        if let Some(closing) = &mut reader.closing {
            return match closing.as_mut().poll(context) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(error)) => {
                    reader.closing = None;
                    reader.completed = true;
                    Poll::Ready(Err(io::Error::other(error)))
                }
                Poll::Ready(Ok(())) => {
                    reader.closing = None;
                    reader.completed = true;
                    Poll::Ready(reader.upstream_error.take().map_or(Ok(()), Err))
                }
            };
        }

        let filled_before = buffer.filled().len();
        match reader.inner.as_mut().poll_read(context, buffer) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(error)) => {
                reader.upstream_error = Some(error);
                reader.begin_close();
                context.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Ready(Ok(())) if buffer.filled().len() == filled_before => {
                reader.begin_close();
                context.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
        }
    }
}
