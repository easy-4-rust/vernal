//! 事务定义。

/// 事务传播行为。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Propagation {
    /// 如果当前存在事务则加入，否则新建一个事务。
    Required,
    /// 总是新建一个事务，并挂起当前事务（如果存在）。
    RequiresNew,
    /// 必须在已有事务中执行，否则抛出异常。
    Mandatory,
    /// 使用保存点在当前事务内嵌套执行。
    Nested,
    /// 如果当前存在事务则加入，否则以非事务方式执行。
    Supports,
    /// 以非事务方式执行，并挂起当前事务（如果存在）。
    NotSupported,
    /// 必须以非事务方式执行，否则抛出异常。
    Never,
}

/// 事务隔离级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Isolation {
    /// 使用底层数据源的默认隔离级别。
    Default,
    /// 读未提交，允许脏读。
    ReadUncommitted,
    /// 读已提交，防止脏读。
    ReadCommitted,
    /// 可重复读，防止脏读和不可重复读。
    RepeatableRead,
    /// 可串行化，最高隔离级别。
    Serializable,
}

/// 事务定义。
#[derive(Debug, Clone)]
pub struct TransactionDefinition {
    /// 事务传播行为。
    pub propagation: Propagation,
    /// 事务隔离级别。
    pub isolation: Isolation,
    /// 是否为只读事务。
    pub read_only: bool,
    /// 事务超时时间（秒），0 表示不超时。
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
