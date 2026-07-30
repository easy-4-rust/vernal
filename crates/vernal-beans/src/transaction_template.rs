//! TransactionTemplate — 事务模板。
use crate::transaction::Transaction;
use crate::transaction_manager::TransactionManager;

/// 事务模板。
#[derive(Clone, Debug)]
pub struct TransactionTemplate;
impl TransactionTemplate {
    pub fn new() -> Self { Self }
    pub fn execute<F, R>(&self, f: F) -> Result<R, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce() -> Result<R, Box<dyn std::error::Error + Send + Sync>>,
    {
        f()
    }
}
