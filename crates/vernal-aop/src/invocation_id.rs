//! 调用标识对象。

use std::{
    fmt,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_INVOCATION_ID: AtomicU64 = AtomicU64::new(1);

/// 进程内单调递增的调用标识。
///
/// 该标识用于日志、指标和链路关联，不承担安全随机数职责。使用无锁原子计数器
/// 可以避免为每次调用引入 UUID 生成依赖。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InvocationId(u64);

impl InvocationId {
    /// 分配新的调用标识。
    #[must_use]
    pub fn next() -> Self {
        Self(NEXT_INVOCATION_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// 返回底层数值。
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl Default for InvocationId {
    fn default() -> Self {
        Self::next()
    }
}

impl fmt::Display for InvocationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
