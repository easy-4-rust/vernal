//! Tower AOP 服务错误对象。

use std::{error::Error, fmt};

use vernal_aop::InvocationError;

/// `AopService` 可观察的结构化错误。
#[derive(Debug)]
#[non_exhaustive]
pub enum AopServiceError<E> {
    /// 请求没有经过 `VernalLayer`，因此缺少应用上下文。
    MissingApplicationContext,
    /// 请求没有经过 `RequestScopeLayer`，因此缺少请求作用域。
    MissingRequestScope,
    /// 请求既没有现成的请求上下文，路由解析器也无法生成路由元数据。
    MissingRouteMetadata,
    /// AOP 调用链执行失败。
    Invocation(InvocationError),
    /// 擦除后的响应信封已被重复消费。
    ResponseAlreadyTaken,
    /// 下游 Tower 服务返回的原生错误。
    Upstream(E),
}

impl<E> fmt::Display for AopServiceError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingApplicationContext => {
                formatter.write_str("application context missing from request extensions")
            }
            Self::MissingRequestScope => {
                formatter.write_str("web request scope missing from request extensions")
            }
            Self::MissingRouteMetadata => {
                formatter.write_str("route metadata could not be resolved")
            }
            Self::Invocation(source) => write!(formatter, "AOP invocation failed: {source}"),
            Self::ResponseAlreadyTaken => {
                formatter.write_str("tower response envelope was already consumed")
            }
            Self::Upstream(source) => write!(formatter, "tower upstream service failed: {source}"),
        }
    }
}

impl<E> Error for AopServiceError<E>
where
    E: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Invocation(source) => Some(source),
            Self::Upstream(source) => Some(source),
            _ => None,
        }
    }
}
