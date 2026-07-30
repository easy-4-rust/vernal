//! ScheduledExecutor — 调度执行器。
use std::sync::Arc;

/// 调度执行器。
#[derive(Clone, Debug, Default)]
pub struct ScheduledExecutor;
impl ScheduledExecutor {
    pub fn new() -> Self { Self }
    pub fn schedule(&self, _task: Arc<dyn Fn() + Send + Sync>, _delay_ms: u64) {}
    pub fn schedule_at_fixed_rate(&self, _task: Arc<dyn Fn() + Send + Sync>, _period_ms: u64) {}
}
