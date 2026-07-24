//! Tower Service 错误映射 Service 对象。

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use tower::Service;

use crate::TowerErrorMapper;

/// 在不改变请求和成功响应类型的前提下执行可配置错误映射。
///
/// Tower 允许 `poll_ready` 在拿到请求前返回错误。本对象保存该一次性结构化错误，
/// 在紧接着的 `call` 中执行 Mapper 并跳过下游。缓存错误而非响应，可兼容 Axum
/// Body 等 `Send` 但非 `Sync` 的原生响应。克隆 Service 时不复制待处理错误，
/// 避免同一 readiness 失败被多个克隆重复消费。
pub struct ErrorMappingService<S, M>
where
    M: TowerErrorMapper,
{
    inner: S,
    mapper: M,
    readiness_error: Option<M::Source>,
}

impl<S, M> ErrorMappingService<S, M>
where
    M: TowerErrorMapper,
{
    /// 由 `ErrorMappingLayer` 创建映射 Service。
    pub(crate) const fn new(inner: S, mapper: M) -> Self {
        Self {
            inner,
            mapper,
            readiness_error: None,
        }
    }
}

impl<S, M> Clone for ErrorMappingService<S, M>
where
    S: Clone,
    M: TowerErrorMapper,
{
    fn clone(&self) -> Self {
        Self::new(self.inner.clone(), self.mapper.clone())
    }
}

impl<S, M, Request> Service<Request> for ErrorMappingService<S, M>
where
    S: Service<Request, Response = M::Response, Error = M::Source> + Clone + Send + 'static,
    S::Future: Send + 'static,
    M: TowerErrorMapper,
    M::Response: Send + 'static,
    M::Source: Send + 'static,
    M::Error: Send + 'static,
    Request: Send + 'static,
{
    type Response = M::Response;
    type Error = M::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, context: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.readiness_error.is_some() {
            return Poll::Ready(Ok(()));
        }

        match self.inner.poll_ready(context) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(error)) => {
                self.readiness_error = Some(error);
                Poll::Ready(Ok(()))
            }
        }
    }

    fn call(&mut self, request: Request) -> Self::Future {
        if let Some(error) = self.readiness_error.take() {
            let mapper = self.mapper.clone();
            return Box::pin(async move {
                drop(request);
                mapper.map_error(error)
            });
        }

        // 调用刚刚通过 readiness 的原始实例；映射策略和值语义响应随 Future
        // 移动，不需要共享锁，也不会让 `&mut self` 借用跨越 await。
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let mapper = self.mapper.clone();
        Box::pin(async move {
            match inner.call(request).await {
                Ok(response) => Ok(response),
                Err(error) => mapper.map_error(error),
            }
        })
    }
}
