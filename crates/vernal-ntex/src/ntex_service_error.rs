//! Ntex 原生 Service 错误信封对象。

use std::{error::Error, fmt};

/// 让只要求 Display/Debug 的 Ntex 错误穿过 Local-AOP 错误通道后原样恢复。
pub(crate) struct NtexServiceError<E> {
    inner: E,
}

impl<E> NtexServiceError<E> {
    /// 包装原生 Service 错误。
    pub(crate) const fn new(inner: E) -> Self {
        Self { inner }
    }

    /// 恢复原生 Service 错误。
    pub(crate) fn into_inner(self) -> E {
        self.inner
    }
}

impl<E> fmt::Debug for NtexServiceError<E>
where
    E: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("NtexServiceError")
            .field(&self.inner)
            .finish()
    }
}

impl<E> fmt::Display for NtexServiceError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Ntex target service failed: {}", self.inner)
    }
}

impl<E> Error for NtexServiceError<E> where E: fmt::Debug + fmt::Display + 'static {}
