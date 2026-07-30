//! TaskScheduler — 任务调度器 trait。
use std::fmt;

/// 任务调度器 trait。
pub trait TaskScheduler: Send + Sync + fmt::Debug {
    fn schedule(&self, task: Box<dyn FnOnce() + Send>, delay_ms: u64) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
