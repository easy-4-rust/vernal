//! `ApplicationContext` 注入 Service 对象。

use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use http::Request;
use tower::Service;
use vernal_context::ApplicationContext;

/// 在调用下游 Service 前写入显式 `ApplicationContext` Extension。
#[derive(Clone)]
pub struct VernalService<S> {
    inner: S,
    context: Arc<ApplicationContext>,
}

impl<S> VernalService<S> {
    /// 创建 Context 注入 Service。
    pub(crate) fn new(inner: S, context: Arc<ApplicationContext>) -> Self {
        Self { inner, context }
    }
}

impl<S, B> Service<Request<B>> for VernalService<S>
where
    S: Service<Request<B>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, context: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(context)
    }

    fn call(&mut self, mut request: Request<B>) -> Self::Future {
        request.extensions_mut().insert(Arc::clone(&self.context));
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        Box::pin(async move { inner.call(request).await })
    }
}
