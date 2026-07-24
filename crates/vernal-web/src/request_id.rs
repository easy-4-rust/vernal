//! Web 请求标识对象。

use std::{
    fmt,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// 进程内单调递增的请求标识。
///
/// Adapter 可以用上游 Trace ID 替代日志关联值，但该标识始终可用，且不包含
/// 用户输入，不会制造无限基数标签。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RequestId(u64);

impl RequestId {
    /// 分配新请求标识。
    #[must_use]
    pub fn next() -> Self {
        Self(NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// 返回底层数值。
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::next()
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
