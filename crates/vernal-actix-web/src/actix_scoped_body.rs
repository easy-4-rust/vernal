//! Actix Web 请求 Scope 感知响应 Body 对象。

use std::{
    error::Error,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use actix_web::{
    body::{BodySize, MessageBody},
    web::Bytes,
};
use tokio_util::sync::{CancellationToken, DropGuard};
use vernal_web::{ScopeError, WebRequestScope};

use crate::ActixBodyError;

type CloseFuture = Pin<Box<dyn Future<Output = Result<(), ScopeError>> + 'static>>;

/// 将 Actix 响应 Body 生命周期与 `WebRequestScope` 绑定。
pub struct ActixScopedBody<B>
where
    B: MessageBody,
{
    inner: Pin<Box<B>>,
    scope: Arc<WebRequestScope>,
    drop_guard: Option<DropGuard>,
    closing: Option<CloseFuture>,
    upstream_error: Option<Box<dyn Error>>,
    completed: bool,
}

impl<B> ActixScopedBody<B>
where
    B: MessageBody,
{
    /// 包装 Actix 原生响应 Body。
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

    /// 启动显式异步 Scope 关闭。
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

impl<B> MessageBody for ActixScopedBody<B>
where
    B: MessageBody + 'static,
{
    type Error = ActixBodyError;

    fn size(&self) -> BodySize {
        self.inner.size()
    }

    fn poll_next(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
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
                    Poll::Ready(Some(Err(ActixBodyError::Scope(error))))
                }
                Poll::Ready(Ok(())) => {
                    body.closing = None;
                    body.completed = true;
                    match body.upstream_error.take() {
                        Some(error) => Poll::Ready(Some(Err(ActixBodyError::Upstream(error)))),
                        None => Poll::Ready(None),
                    }
                }
            };
        }

        match body.inner.as_mut().poll_next(context) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(Ok(bytes))) => Poll::Ready(Some(Ok(bytes))),
            Poll::Ready(Some(Err(error))) => {
                body.upstream_error = Some(error.into());
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
}
