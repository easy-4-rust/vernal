//! TransactionManager — 事务管理器。
use crate::transaction::Transaction;

/// 事务管理器 trait。
pub trait TransactionManager: Send + Sync {
    fn begin(&self) -> Result<Box<dyn Transaction>, Box<dyn std::error::Error + Send + Sync>>;
    fn commit(&self, transaction: Box<dyn Transaction>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn rollback(&self, transaction: Box<dyn Transaction>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
