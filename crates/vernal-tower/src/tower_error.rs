//! Tower 请求执行错误对象。

use std::{error::Error, fmt};

use vernal_web::ScopeError;

/// 区分下游 Service 错误与请求 Scope 关闭错误。
#[derive(Debug)]
pub enum TowerError<E> {
    /// 下游 Service 返回错误。
    Upstream(E),
    /// 请求作用域关闭失败。
    Scope(ScopeError),
}

impl<E> fmt::Display for TowerError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Upstream(error) => write!(formatter, "tower upstream error: {error}"),
            Self::Scope(error) => write!(formatter, "tower request scope error: {error}"),
        }
    }
}

impl<E> Error for TowerError<E>
where
    E: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Upstream(error) => Some(error),
            Self::Scope(error) => Some(error),
        }
    }
}
