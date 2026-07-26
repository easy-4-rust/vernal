//! 事务定义。

/// 事务传播行为。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Propagation {
    Required,
    RequiresNew,
    Mandatory,
    Nested,
    Supports,
    NotSupported,
    Never,
}

/// 事务隔离级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Isolation {
    Default,
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

/// 事务定义。
#[derive(Debug, Clone)]
pub struct TransactionDefinition {
    pub propagation: Propagation,
    pub isolation: Isolation,
    pub read_only: bool,
    pub timeout_secs: u64,
}

impl Default for TransactionDefinition {
    fn default() -> Self {
        Self {
            propagation: Propagation::Required,
            isolation: Isolation::Default,
            read_only: false,
            timeout_secs: 0,
        }
    }
}
