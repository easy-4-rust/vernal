//! 异步任务执行器 trait。
//!
//! 对标 Spring `org.springframework.core.task.AsyncTaskExecutor`。

use super::task_error::TaskError;
use super::task_executor::TaskExecutor;

/// 异步任务执行器 trait。
///
/// 对应 Java: org.springframework.core.task.AsyncTaskExecutor
pub trait AsyncTaskExecutor: TaskExecutor {
    /// 异步执行给定的 Future，返回 JoinHandle。
    ///
    /// 对应 Java: `AsyncTaskExecutor#submit(Callable)`
    fn execute_async(
        &self,
        task: impl FnOnce() + Send + 'static,
    ) -> impl std::future::Future<Output = Result<(), TaskError>> + Send;
}
