//! 简单任务执行器（同步）。
//!
//! 对标 Spring `org.springframework.core.task.SyncTaskExecutor`。

use super::task_executor::TaskExecutor;

/// 简单同步任务执行器。
///
/// 对应 Java: org.springframework.core.task.SyncTaskExecutor
#[derive(Debug, Default)]
pub struct SimpleTaskExecutor {
    name: String,
}

impl SimpleTaskExecutor {
    /// 创建默认执行器。
    #[must_use]
    pub fn new() -> Self {
        Self { name: "simple".to_string() }
    }

    /// 创建带名称的执行器。
    #[must_use]
    pub fn with_name(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl TaskExecutor for SimpleTaskExecutor {
    fn execute(&self, task: Box<dyn FnOnce() + Send + 'static>) {
        task();
    }

    fn name(&self) -> &str {
        &self.name
    }
}
