//! Poem 请求 Scope 感知响应字节流对象。

use std::{
    future::Future,
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures_core::Stream;
use tokio_util::sync::{CancellationToken, DropGuard};
use vernal_web::{ScopeError, WebRequestScope};

type CloseFuture = Pin<Box<dyn Future<Output = Result<(), ScopeError>> + Send + 'static>>;

/// 将 Poem 原生响应字节流生命周期与 `WebRequestScope` 绑定。
///
/// 正常完成和上游错误都会先等待异步 Scope 关闭。若响应消费 Future 被丢弃，
/// `DropGuard` 会发出同步取消信号，由 Endpoint 预先启动的清理任务完成异步关闭。
pub struct PoemScopedStream<S>
where
    S: Stream<Item = Result<Bytes, io::Error>>,
{
    inner: Pin<Box<S>>,
    scope: Arc<WebRequestScope>,
    drop_guard: Option<DropGuard>,
    closing: Option<CloseFuture>,
    upstream_error: Option<io::Error>,
    completed: bool,
}

impl<S> PoemScopedStream<S>
where
    S: Stream<Item = Result<Bytes, io::Error>>,
{
    /// 包装 Poem 原生响应字节流。
    #[must_use]
    pub fn new(inner: S, scope: Arc<WebRequestScope>, cancellation: CancellationToken) -> Self {
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

        // 显式完成路径解除 Drop 兜底，但不抢先取消。`scope.close()` 会在获得
        // Scope 操作锁后发出取消，使关闭钩子错误能够由当前响应流稳定上报。
        if let Some(drop_guard) = self.drop_guard.take() {
            drop_guard.disarm();
        }
        let scope = Arc::clone(&self.scope);
        self.closing = Some(Box::pin(async move { scope.close().await }));
    }
}

impl<S> Stream for PoemScopedStream<S>
where
    S: Stream<Item = Result<Bytes, io::Error>> + Send + 'static,
{
    type Item = Result<Bytes, io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let stream = self.as_mut().get_mut();
        if stream.completed {
            return Poll::Ready(None);
        }

        if let Some(closing) = &mut stream.closing {
            return match closing.as_mut().poll(context) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(error)) => {
                    stream.closing = None;
                    stream.completed = true;
                    Poll::Ready(Some(Err(io::Error::other(error))))
                }
                Poll::Ready(Ok(())) => {
                    stream.closing = None;
                    stream.completed = true;
                    Poll::Ready(stream.upstream_error.take().map(Err))
                }
            };
        }

        match stream.inner.as_mut().poll_next(context) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(Ok(bytes))) => Poll::Ready(Some(Ok(bytes))),
            Poll::Ready(Some(Err(error))) => {
                stream.upstream_error = Some(error);
                stream.begin_close();
                context.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Ready(None) => {
                stream.begin_close();
                context.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}
