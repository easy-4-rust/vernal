//! Tide 请求 Scope 感知响应 Reader 对象。

use std::{
    future::Future,
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures_lite::io::AsyncRead;
use tide::Body;
use tokio::task::JoinHandle;
use tokio_util::sync::{CancellationToken, DropGuard};
use vernal_web::{ScopeError, WebRequestScope};

use crate::TideBodyError;

/// 将 Tide 字节流响应生命周期与 `WebRequestScope` 绑定。
///
/// Tide 的公共 Body 只暴露 `AsyncBufRead`，没有 HTTP Frame/Trailer API。
/// 本对象因此保留字节、错误、背压和已知长度，但不声明 Trailer 保真。
/// EOF 或上游错误后，它先等待 Tokio 中的异步 Scope 关闭，再向下游报告终止。
pub struct TideScopedReader {
    inner: Body,
    scope: Arc<WebRequestScope>,
    drop_guard: Option<DropGuard>,
    closing: Option<JoinHandle<Result<(), ScopeError>>>,
    upstream_error: Option<io::Error>,
    completed: bool,
}

impl TideScopedReader {
    /// 包装 Tide 原生响应 Body。
    #[must_use]
    pub fn new(inner: Body, scope: Arc<WebRequestScope>, cancellation: CancellationToken) -> Self {
        Self {
            inner,
            scope,
            drop_guard: Some(cancellation.drop_guard()),
            closing: None,
            upstream_error: None,
            completed: false,
        }
    }

    /// 在 Vernal 官方 Tokio runtime 上启动显式 Scope 关闭。
    fn begin_close(&mut self) {
        if self.closing.is_some() || self.completed {
            return;
        }
        if let Some(drop_guard) = self.drop_guard.take() {
            drop_guard.disarm();
        }
        let scope = Arc::clone(&self.scope);
        self.closing = Some(tokio::spawn(async move { scope.close().await }));
    }

    /// 轮询异步关闭，并恢复 EOF 或原始上游错误。
    fn poll_close(&mut self, context: &mut Context<'_>) -> Poll<io::Result<usize>> {
        let Some(closing) = &mut self.closing else {
            return Poll::Pending;
        };
        match Pin::new(closing).poll(context) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(error)) => {
                self.closing = None;
                self.completed = true;
                Poll::Ready(Err(io::Error::other(TideBodyError::CleanupTask(error))))
            }
            Poll::Ready(Ok(Err(error))) => {
                self.closing = None;
                self.completed = true;
                Poll::Ready(Err(io::Error::other(TideBodyError::Scope(error))))
            }
            Poll::Ready(Ok(Ok(()))) => {
                self.closing = None;
                self.completed = true;
                match self.upstream_error.take() {
                    Some(error) => Poll::Ready(Err(error)),
                    None => Poll::Ready(Ok(0)),
                }
            }
        }
    }
}

impl AsyncRead for TideScopedReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        if self.completed {
            return Poll::Ready(Ok(0));
        }
        if self.closing.is_some() {
            return self.poll_close(context);
        }

        match Pin::new(&mut self.inner).poll_read(context, buffer) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(0)) => {
                self.begin_close();
                context.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Ready(Ok(read)) => Poll::Ready(Ok(read)),
            Poll::Ready(Err(error)) => {
                self.upstream_error = Some(error);
                self.begin_close();
                context.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}
