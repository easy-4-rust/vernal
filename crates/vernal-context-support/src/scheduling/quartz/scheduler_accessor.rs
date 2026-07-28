//! 调度器访问器 — 对标 `SchedulerAccessor`。

use std::sync::Arc;

/// 调度器访问器 trait。
pub trait SchedulerAccessor: Send + Sync {
    /// 获取调度器。
    fn get_scheduler(&self) -> Option<Arc<dyn std::any::Any + Send + Sync>>;
    /// 注册任务和触发器。
    fn register_jobs_and_triggers(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    /// 注册监听器。
    fn register_listeners(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    /// 是否覆盖已有任务。
    fn overwrite_existing_jobs(&self) -> bool;
    /// 设置是否覆盖已有任务。
    fn set_overwrite_existing_jobs(&mut self, overwrite: bool);
}
