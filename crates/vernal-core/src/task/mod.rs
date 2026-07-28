//! 任务执行器抽象模块。
//!
//! 对标 Spring `org.springframework.core.task.TaskExecutor`。
//!
//! # 与 Spring 的对应关系
//!
//! | Spring | vernal-core |
//! |---|---|
//! | `TaskExecutor` interface | `TaskExecutor` trait |
//! | `AsyncTaskExecutor` interface | `AsyncTaskExecutor` trait |
//! | `SimpleAsyncTaskExecutor` | `SimpleTaskExecutor` struct |

/// 任务执行器 trait。
///
/// 对标 Spring `org.springframework.core.task.TaskExecutor`。
///
/// # 示例
///
/// ```rust
/// use vernal_core::task::{TaskExecutor, SimpleTaskExecutor};
///
/// let executor = SimpleTaskExecutor::new();
/// executor.execute(|| {
///     // 执行任务
/// });
/// ```
pub trait TaskExecutor: Send + Sync {
    /// 执行任务。
    fn execute(&self, task: impl FnOnce() + Send + 'static);

    /// 获取执行器名称。
    fn name(&self) -> &str;
}

/// 异步任务执行器 trait。
///
/// 对标 Spring `org.springframework.core.task.AsyncTaskExecutor`。

pub trait AsyncTaskExecutor: TaskExecutor {
    /// 异步执行任务。
    async fn execute_async(&self, task: impl FnOnce() + Send + 'static) -> Result<(), TaskError>;
}

/// 任务执行错误。
#[derive(Debug, Clone)]
pub enum TaskError {
    /// 任务被拒绝
    Rejected(String),
    /// 任务执行超时
    Timeout(String),
    /// 任务执行失败
    ExecutionFailed(String),
}

impl std::fmt::Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rejected(msg) => write!(f, "Task rejected: {msg}"),
            Self::Timeout(msg) => write!(f, "Task timeout: {msg}"),
            Self::ExecutionFailed(msg) => write!(f, "Task execution failed: {msg}"),
        }
    }
}

impl std::error::Error for TaskError {}

/// 简单任务执行器（同步）。
///
/// 对标 Spring `org.springframework.core.task.SyncTaskExecutor`。
#[derive(Debug, Clone)]
pub struct SimpleTaskExecutor {
    name: String,
}

impl SimpleTaskExecutor {
    /// 创建新的简单任务执行器。
    pub fn new() -> Self {
        Self {
            name: "SimpleTaskExecutor".to_string(),
        }
    }

    /// 创建带名称的简单任务执行器。
    pub fn with_name(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Default for SimpleTaskExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskExecutor for SimpleTaskExecutor {
    fn execute(&self, task: impl FnOnce() + Send + 'static) {
        task();
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[test]
    fn simple_task_executor_executes_task() {
        let executor = SimpleTaskExecutor::new();
        let executed = Arc::new(AtomicBool::new(false));
        let executed_clone = executed.clone();

        executor.execute(move || {
            executed_clone.store(true, Ordering::Relaxed);
        });

        assert!(executed.load(Ordering::Relaxed));
    }

    #[test]
    fn simple_task_executor_name() {
        let executor = SimpleTaskExecutor::new();
        assert_eq!(executor.name(), "SimpleTaskExecutor");

        let executor = SimpleTaskExecutor::with_name("MyExecutor");
        assert_eq!(executor.name(), "MyExecutor");
    }

    #[test]
    fn task_error_display() {
        let err = TaskError::Rejected("queue full".to_string());
        assert!(err.to_string().contains("Task rejected"));
    }

    #[test]
    fn task_error_is_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<TaskError>();
    }
}
