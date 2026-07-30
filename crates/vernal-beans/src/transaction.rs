//! Transaction — 事务系统。
use std::fmt;

/// 事务 trait。
pub trait Transaction: Send + Sync + fmt::Debug {
    fn commit(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn rollback(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn is_active(&self) -> bool;
    fn is_read_only(&self) -> bool { false }
}

/// 事务状态。
#[derive(Clone, Debug, PartialEq)]
pub enum TransactionStatus {
    Active,
    Committed,
    RolledBack,
}
