//! Salvo 请求 Scope 感知响应 Body 对象。

use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use bytes::Bytes;
use http_body::{Body, Frame, SizeHint};
use salvo::{BoxedError, http::ResBody};
use sync_wrapper::SyncWrapper;
use tokio_util::sync::{CancellationToken, DropGuard};
use vernal_web::{ScopeError, WebRequestScope};

type CloseFuture = Pin<Box<dyn Future<Output = Result<(), ScopeError>> + Send + 'static>>;

/// 将 Salvo 原生响应 Body 生命周期与 `WebRequestScope` 绑定。
///
/// `ResBody` 直接实现标准 `http_body::Body`，因此本对象保留 Data Frame、
/// Trailer、背压与上游错误，不需要把 Salvo 响应复制到 Vernal HTTP 模型。
pub struct SalvoScopedBody {
    inner: Pin<Box<ResBody>>,
    scope: Arc<WebRequestScope>,
    drop_guard: Option<DropGuard>,
    closing: SyncWrapper<Option<CloseFuture>>,
    upstream_error: Option<BoxedError>,
    completed: bool,
}

impl SalvoScopedBody {
    /// 包装 Salvo 原生响应 Body。
    #[must_use]
    pub fn new(
        inner: ResBody,
        scope: Arc<WebRequestScope>,
        cancellation: CancellationToken,
    ) -> Self {
        Self {
            inner: Box::pin(inner),
            scope,
            drop_guard: Some(cancellation.drop_guard()),
            closing: SyncWrapper::new(None),
            upstream_error: None,
            completed: false,
        }
    }

    /// 启动显式 Scope 关闭 Future。
    fn begin_close(&mut self) {
        if self.closing.get_mut().is_some() || self.completed {
            return;
        }

        // 显式完成路径解除 Drop 兜底，但不抢先取消。`scope.close()` 在持有
        // Scope 操作锁后发出取消，保证关闭钩子错误由当前 Body 稳定上报。
        if let Some(drop_guard) = self.drop_guard.take() {
            drop_guard.disarm();
        }
        let scope = Arc::clone(&self.scope);
        *self.closing.get_mut() = Some(Box::pin(async move { scope.close().await }));
    }
}

impl Body for SalvoScopedBody {
    type Data = Bytes;
    type Error = BoxedError;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let body = self.as_mut().get_mut();

        if let Some(closing) = body.closing.get_mut() {
            return match closing.as_mut().poll(context) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(error)) => {
                    *body.closing.get_mut() = None;
                    body.completed = true;
                    Poll::Ready(Some(Err(Box::new(error))))
                }
                Poll::Ready(Ok(())) => {
                    *body.closing.get_mut() = None;
                    body.completed = true;
                    Poll::Ready(body.upstream_error.take().map(Err))
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
