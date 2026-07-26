//! 事务状态。

/// 事务状态。
#[derive(Debug)]
pub struct TransactionStatus {
    /// 是否只读
    pub read_only: bool,
    /// 是否已完成
    pub completed: bool,
    /// 是否标记为回滚
    pub rollback_only: bool,
}

impl TransactionStatus {
    /// 创建新的事务状态。
    #[must_use]
    pub fn new(read_only: bool) -> Self {
        Self {
            read_only,
            completed: false,
            rollback_only: false,
        }
    }

    /// 标记为仅回滚。
    pub fn set_rollback_only(&mut self) {
        self.rollback_only = true;
    }

    /// 是否标记为仅回滚。
    #[must_use]
    pub fn is_rollback_only(&self) -> bool {
        self.rollback_only
    }

    /// 是否已完成。
    #[must_use]
    pub fn is_completed(&self) -> bool {
        self.completed
    }
}
