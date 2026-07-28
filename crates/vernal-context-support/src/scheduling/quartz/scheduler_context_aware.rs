//! 调度器上下文感知 — 对标 `SchedulerContextAware`。

/// 调度器上下文数据。
#[derive(Debug, Clone, Default)]
pub struct SchedulerContext {
    pub data: std::collections::HashMap<String, String>,
}

/// 调度器上下文感知 trait。
pub trait SchedulerContextAware: Send + Sync {
    fn set_scheduler_context(&mut self, context: SchedulerContext);
}
