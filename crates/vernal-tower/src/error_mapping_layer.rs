//! Tower Service 错误映射 Layer 对象。

use tower::Layer;

use crate::{ErrorMappingService, TowerErrorMapper};

/// 使用调用方提供的映射策略统一处理 Tower readiness 与 call 错误。
///
/// Mapper 可以把错误恢复成下游原生响应，也可以保留或转换成新的
/// `Service::Error`。Layer 不解释 HTTP 状态、gRPC Code 或业务错误内容，协议语义
/// 继续由 Axum、Tonic、Warp 等具体 Adapter 持有。
#[derive(Clone)]
pub struct ErrorMappingLayer<M> {
    mapper: M,
}

impl<M> ErrorMappingLayer<M> {
    /// 创建显式错误映射 Layer。
    #[must_use]
    pub const fn new(mapper: M) -> Self {
        Self { mapper }
    }
}

impl<S, M> Layer<S> for ErrorMappingLayer<M>
where
    M: TowerErrorMapper,
{
    type Service = ErrorMappingService<S, M>;

    fn layer(&self, inner: S) -> Self::Service {
        ErrorMappingService::new(inner, self.mapper.clone())
    }
}
