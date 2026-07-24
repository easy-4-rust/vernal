//! Tower 请求上下文传播错误对象。

use std::{error::Error, fmt};

/// `ContextPropagationService` 可观察的结构化错误。
///
/// 传播层只负责验证请求作用域和路由身份，不把框架中立错误提前转换成 HTTP
/// 响应。上层 Axum、Tonic 或其他 Tower Adapter 可以在自己的原生错误边界完成
/// 状态码映射，同时保留下游 Service 的原始错误类型。
#[derive(Debug)]
#[non_exhaustive]
pub enum ContextPropagationError<E> {
    /// 请求没有经过 `RequestScopeLayer` 或等价的框架原生 Scope 中间件。
    MissingRequestScope,
    /// 请求没有现成 `RequestContext`，路由解析器也无法产生稳定路由元数据。
    MissingRouteMetadata,
    /// 下游 Tower Service 返回的原生错误。
    Upstream(E),
}

impl<E> fmt::Display for ContextPropagationError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRequestScope => {
                formatter.write_str("web request scope missing from request extensions")
            }
            Self::MissingRouteMetadata => {
                formatter.write_str("route metadata could not be resolved")
            }
            Self::Upstream(source) => {
                write!(formatter, "tower upstream service failed: {source}")
            }
        }
    }
}

impl<E> Error for ContextPropagationError<E>
where
    E: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Upstream(source) => Some(source),
            _ => None,
        }
    }
}
