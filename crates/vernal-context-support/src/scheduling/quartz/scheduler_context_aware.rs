//! 调度器上下文感知 — 对标 `SchedulerContextAware`。

/// 调度器上下文数据。
#[derive(Debug, Clone, Default)]
pub struct SchedulerContext {
    /// 上下文数据。
    pub data: std::collections::HashMap<String, String>,
}

/// 调度器上下文感知 trait。
pub trait SchedulerContextAware: Send + Sync {
    /// 设置调度器上下文。
    fn set_scheduler_context(&mut self, context: SchedulerContext);
}
