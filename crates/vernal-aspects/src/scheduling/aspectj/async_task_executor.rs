//! 对标 `org.springframework.core.task.AsyncTaskExecutor` 接口。
//!
//! 异步任务执行器：提交异步任务并返回 Future。

use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// 异步任务执行结果。
#[derive(Debug)]
pub enum AsyncTaskResult {
    /// 正常完成。
    Ok(Box<dyn Any + Send + Sync>),
    /// 异常完成。
    Err(String),
}

/// 异步任务执行器 trait。
///
/// 对标 Spring 的 `AsyncTaskExecutor` 接口。
/// 负责提交异步任务并返回 Future。
pub trait AsyncTaskExecutor: Send + Sync + 'static {
    /// 提交异步任务。
    ///
    /// 对应 Spring 的 `AsyncTaskExecutor#submit(Callable<T>)`。
    fn submit(
        &self,
        task: Pin<Box<dyn Future<Output = AsyncTaskResult> + Send>>,
    ) -> Pin<Box<dyn Future<Output = AsyncTaskResult> + Send>>;

    /// 提交多个异步任务。
    fn submit_all(
        &self,
        tasks: Vec<Pin<Box<dyn Future<Output = AsyncTaskResult> + Send>>>,
    ) -> Vec<Pin<Box<dyn Future<Output = AsyncTaskResult> + Send>>> {
        tasks.into_iter().map(|t| self.submit(t)).collect()
    }

    /// 获取执行器名称。
    fn get_executor_name(&self) -> &str;
}

/// 默认的阻塞执行器（不真正异步，用于测试或回退）。
pub struct DefaultAsyncTaskExecutor {
    name: String,
}

impl DefaultAsyncTaskExecutor {
    /// 创建默认执行器。
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Default for DefaultAsyncTaskExecutor {
    fn default() -> Self {
        Self::new("default")
    }
}

impl AsyncTaskExecutor for DefaultAsyncTaskExecutor {
    fn submit(
        &self,
        task: Pin<Box<dyn Future<Output = AsyncTaskResult> + Send>>,
    ) -> Pin<Box<dyn Future<Output = AsyncTaskResult> + Send>> {
        // 简单实现：直接在当前线程执行
        task
    }

    fn get_executor_name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_async_task_executor_creation() {
        let executor = DefaultAsyncTaskExecutor::new("test-executor");
        assert_eq!(executor.get_executor_name(), "test-executor");
    }

    #[test]
    fn test_default_async_task_executor_default() {
        let executor = DefaultAsyncTaskExecutor::default();
        assert_eq!(executor.get_executor_name(), "default");
    }

    #[test]
    fn test_async_task_executor_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<DefaultAsyncTaskExecutor>();
        assert_sync::<DefaultAsyncTaskExecutor>();
    }
}
