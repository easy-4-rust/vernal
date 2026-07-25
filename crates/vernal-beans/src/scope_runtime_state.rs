//! 自定义作用域内部运行状态对象。

use crate::ScopeState;

/// 在同一互斥区内维护作用域阶段和正在执行的同步操作数量。
pub(crate) struct ScopeRuntimeState {
    pub(crate) state: ScopeState,
    pub(crate) active_operations: usize,
}

impl Default for ScopeRuntimeState {
    fn default() -> Self {
        Self {
            state: ScopeState::Open,
            active_operations: 0,
        }
    }
}
