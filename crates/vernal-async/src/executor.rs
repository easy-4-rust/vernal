//! 异步任务执行器 trait。

use std::future::Future;
use vernal_core::BoxError;

/// 异步任务执行器 trait。
///
/// 对标 Spring 的 `TaskExecutor` / `AsyncTaskExecutor`。
pub trait AsyncTaskExecutor: Send + Sync {
    /// 提交异步任务到执行器。
    fn submit<F>(&self, task: F) -> Result<(), BoxError>
    where
        F: Future<Output = Result<(), BoxError>> + Send + 'static;
}
