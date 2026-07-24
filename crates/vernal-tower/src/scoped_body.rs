//! 请求 Scope 感知的响应 Body 对象。

use std::{
    error::Error,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use bytes::Bytes;
use http_body::{Body, Frame, SizeHint};
use tokio_util::sync::{CancellationToken, DropGuard};
use vernal_web::{ScopeError, WebRequestScope};

use crate::TowerBodyError;

type CloseFuture = Pin<Box<dyn Future<Output = Result<(), ScopeError>> + Send + 'static>>;

/// 将请求 Scope 生命周期延伸到响应 Body 完成、失败或取消。
///
/// Body 正常结束或错误时会先等待异步 Scope 关闭，再向下游报告结束/错误。
/// 如果消费 Future 被直接丢弃，`DropGuard` 只负责同步发出取消信号，预先启动的
/// 清理 task 负责执行异步 close，不在 `Drop` 中运行异步代码。
pub struct ScopedBody<B>
where
    B: Body,
{
    inner: Pin<Box<B>>,
    scope: Arc<WebRequestScope>,
    drop_guard: Option<DropGuard>,
    closing: Option<CloseFuture>,
    upstream_error: Option<Box<B::Error>>,
    completed: bool,
}

impl<B> ScopedBody<B>
where
    B: Body,
{
    /// 包装上游响应 Body。
    ///
    /// 框架适配器可用此构造器把原生 Body 重新包装回框架响应类型，同时复用
    /// Vernal 已验证的请求作用域关闭与取消语义。
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

    /// 启动显式 Scope 关闭 Future。
    fn begin_close(&mut self) {
        if self.closing.is_some() || self.completed {
            return;
        }
        // 显式完成路径解除 Drop 兜底，但暂不直接取消。`scope.close()` 会在持有
        // Scope 操作锁后发出取消，使后台任务只能在显式关闭之后读取幂等结果，
        // 从而保证关闭钩子错误由当前 Body 正确上报。
        if let Some(drop_guard) = self.drop_guard.take() {
            drop_guard.disarm();
        }
        let scope = Arc::clone(&self.scope);
        self.closing = Some(Box::pin(async move { scope.close().await }));
    }
}

impl<B> Body for ScopedBody<B>
where
    B: Body<Data = Bytes> + Send + 'static,
    B::Error: Error + Send + Sync + 'static,
{
    type Data = Bytes;
    type Error = TowerBodyError<B::Error>;

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
                    Poll::Ready(Some(Err(TowerBodyError::Scope(error))))
                }
                Poll::Ready(Ok(())) => {
                    body.closing = None;
                    body.completed = true;
                    match body.upstream_error.take() {
                        Some(error) => Poll::Ready(Some(Err(TowerBodyError::Upstream(*error)))),
                        None => Poll::Ready(None),
                    }
                }
            };
        }

        match body.inner.as_mut().poll_frame(context) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(Ok(frame))) => Poll::Ready(Some(Ok(frame))),
            Poll::Ready(Some(Err(error))) => {
                body.upstream_error = Some(Box::new(error));
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
        self.completed
    }

    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}
