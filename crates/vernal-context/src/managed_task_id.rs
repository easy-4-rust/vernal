//! 受管 Tokio 任务标识对象。

use std::fmt;

/// 标识一个由 [`crate::ManagedTaskSupervisor`] 持有生命周期的后台任务。
///
/// 标识只在所属应用 Context 内有意义，用于测试、诊断和后续精确控制；它不包含
/// 任务地址、业务参数或跨进程稳定语义。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ManagedTaskId(u64);

impl ManagedTaskId {
    /// 由任务监督器分配单调递增标识。
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }

    /// 返回 Context 内部的数值标识。
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ManagedTaskId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
