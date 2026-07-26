//! 事务管理器 trait。

use super::{definition::TransactionDefinition, status::TransactionStatus};

/// 平台事务管理器 trait。
///
/// 对标 Spring 的 `PlatformTransactionManager`。
pub trait PlatformTransactionManager: Send + Sync {
    /// 获取事务。
    fn get_transaction(
        &self,
        definition: &TransactionDefinition,
    ) -> Result<TransactionStatus, TransactionError>;

    /// 提交事务。
    fn commit(&self, status: TransactionStatus) -> Result<(), TransactionError>;

    /// 回滚事务。
    fn rollback(&self, status: TransactionStatus) -> Result<(), TransactionError>;
}

/// 事务错误。
#[derive(Debug, Clone)]
pub struct TransactionError {
    pub message: String,
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "事务错误: {}", self.message)
    }
}

impl std::error::Error for TransactionError {}
