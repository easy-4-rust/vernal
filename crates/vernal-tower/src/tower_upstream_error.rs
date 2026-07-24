//! Tower 下游错误信封对象。

use std::{error::Error, fmt};

/// 标记由下游 `Service` 产生、最终应恢复为原生类型的错误。
///
/// 该包装与拦截器自身返回的业务错误形成明确边界，避免二者恰好使用同一错误
/// 类型时被错误分类。
#[derive(Debug)]
pub(crate) struct TowerUpstreamError<E>(pub(crate) E);

impl<E> fmt::Display for TowerUpstreamError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "tower upstream service failed: {}", self.0)
    }
}

impl<E> Error for TowerUpstreamError<E> where E: Error + 'static {}
