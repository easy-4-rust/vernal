//! 任务执行器 trait。
//!
//! 对标 Spring `org.springframework.core.task.TaskExecutor`。

/// 任务执行器 trait。
///
/// 对应 Java: org.springframework.core.task.TaskExecutor
pub trait TaskExecutor: Send + Sync {
    /// 执行给定的同步任务。
    ///
    /// 对应 Java: `TaskExecutor#execute(Runnable)`
    fn execute(&self, task: Box<dyn FnOnce() + Send + 'static>);

    /// 返回任务执行器名称（用于诊断日志）。
    fn name(&self) -> &str;
}
