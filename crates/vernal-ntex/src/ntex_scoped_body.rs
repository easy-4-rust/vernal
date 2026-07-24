//! Ntex 请求 Scope 感知响应 Body 对象。

use std::{
    error::Error,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll},
};

use ntex::{
    http::body::{Body, BodySize, MessageBody, ResponseBody},
    util::Bytes,
};
use tokio_util::sync::{CancellationToken, DropGuard};
use vernal_web::{ScopeError, WebRequestScope};

use crate::NtexBodyError;

type CloseFuture = Pin<Box<dyn Future<Output = Result<(), ScopeError>> + 'static>>;

/// 将 Ntex 原生响应 Body 的完整消费周期与 `WebRequestScope` 绑定。
///
/// Handler 返回只说明响应头已经生成，并不代表流式 Body 已传输完成。该包装器
/// 在 EOF 或上游错误后异步关闭 Scope；如果客户端提前断开并丢弃 Body，
/// [`DropGuard`] 会发出取消信号，由中间件预先启动的清理任务完成兜底释放。
pub struct NtexScopedBody {
    inner: ResponseBody<Body>,
    scope: Arc<WebRequestScope>,
    drop_guard: Option<DropGuard>,
    closing: Option<CloseFuture>,
    upstream_error: Option<Rc<dyn Error>>,
    completed: bool,
}

impl NtexScopedBody {
    /// 包装 Ntex 原生响应 Body。
    #[must_use]
    pub fn new(
        inner: ResponseBody<Body>,
        scope: Arc<WebRequestScope>,
        cancellation: CancellationToken,
    ) -> Self {
        Self {
            inner,
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

impl MessageBody for NtexScopedBody {
    fn size(&self) -> BodySize {
        self.inner.size()
    }

    fn poll_next_chunk(
        &mut self,
        context: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Rc<dyn Error>>>> {
        if self.completed {
            return Poll::Ready(None);
        }

        if let Some(closing) = &mut self.closing {
            return match closing.as_mut().poll(context) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(error)) => {
                    self.closing = None;
                    self.completed = true;
                    Poll::Ready(Some(Err(Rc::new(NtexBodyError::scope_close(error)))))
                }
                Poll::Ready(Ok(())) => {
                    self.closing = None;
                    self.completed = true;
                    match self.upstream_error.take() {
                        Some(error) => Poll::Ready(Some(Err(error))),
                        None => Poll::Ready(None),
                    }
                }
            };
        }

        match self.inner.poll_next_chunk(context) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(Ok(bytes))) => Poll::Ready(Some(Ok(bytes))),
            Poll::Ready(Some(Err(error))) => {
                self.upstream_error = Some(error);
                self.begin_close();
                context.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Ready(None) => {
                self.begin_close();
                context.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}
