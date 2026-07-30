//! TaskExecutor — 任务执行器 trait。
use std::fmt;

/// 任务执行器 trait。
pub trait TaskExecutor: Send + Sync + fmt::Debug {
    fn execute(&self, task: Box<dyn FnOnce() + Send>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
