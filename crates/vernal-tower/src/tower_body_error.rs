//! Tower 响应 Body 错误对象。

use std::{error::Error, fmt};

use vernal_web::ScopeError;

/// 区分上游 Body 错误与流结束时的 Scope 关闭错误。
#[derive(Debug)]
pub enum TowerBodyError<E> {
    /// 上游 Body 返回错误。
    Upstream(E),
    /// 响应流结束后的请求作用域关闭失败。
    Scope(ScopeError),
}

impl<E> fmt::Display for TowerBodyError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Upstream(error) => write!(formatter, "tower body upstream error: {error}"),
            Self::Scope(error) => write!(formatter, "tower body scope error: {error}"),
        }
    }
}

impl<E> Error for TowerBodyError<E>
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
