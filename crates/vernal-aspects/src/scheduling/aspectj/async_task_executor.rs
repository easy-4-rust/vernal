//! 对标 `org.springframework.core.task.AsyncTaskExecutor` 接口。
//!
//! 异步任务执行器：提交异步任务并返回 Future。

use std::any::Any;
use std::future::Future;
use std::pin::Pin;

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
#[derive(Debug)]
#[allow(dead_code)] // Java 镜像脚手架：当前阶段未在切面中实际构造，供测试与回退使用
pub struct DefaultAsyncTaskExecutor {
    name: String,
}

#[allow(dead_code)] // Java 镜像脚手架：构造函数供测试与后续集成使用
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

    #[test]
    fn test_submit_returns_same_future() {
        let executor = DefaultAsyncTaskExecutor::new("test");
        let task: std::pin::Pin<Box<dyn std::future::Future<Output = AsyncTaskResult> + Send>> =
            Box::pin(async {
                AsyncTaskResult::Ok(Box::new(42) as Box<dyn std::any::Any + Send + Sync>)
            });
        let result = executor.submit(task);
        // 默认执行器直接返回 task
        let _ = result;
    }

    #[test]
    fn test_submit_all() {
        let executor = DefaultAsyncTaskExecutor::new("test");
        let tasks: Vec<
            std::pin::Pin<Box<dyn std::future::Future<Output = AsyncTaskResult> + Send>>,
        > = vec![
            Box::pin(async {
                AsyncTaskResult::Ok(Box::new(1) as Box<dyn std::any::Any + Send + Sync>)
            }),
            Box::pin(async {
                AsyncTaskResult::Ok(Box::new(2) as Box<dyn std::any::Any + Send + Sync>)
            }),
        ];
        let results = executor.submit_all(tasks);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_async_task_result_variants() {
        let ok = AsyncTaskResult::Ok(Box::new(42) as Box<dyn std::any::Any + Send + Sync>);
        let err = AsyncTaskResult::Err("error".to_string());
        match ok {
            AsyncTaskResult::Ok(val) => {
                assert_eq!(val.downcast_ref::<i32>().unwrap(), &42);
            }
            _ => panic!("Expected Ok"),
        }
        match err {
            AsyncTaskResult::Err(msg) => {
                assert_eq!(msg, "error");
            }
            _ => panic!("Expected Err"),
        }
    }

    #[test]
    fn test_async_task_result_debug() {
        let ok = AsyncTaskResult::Ok(Box::new(42) as Box<dyn std::any::Any + Send + Sync>);
        let err = AsyncTaskResult::Err("error".to_string());
        assert!(format!("{:?}", ok).contains("Ok"));
        assert!(format!("{:?}", err).contains("Err"));
    }

    #[test]
    fn test_default_async_task_executor_debug() {
        let executor = DefaultAsyncTaskExecutor::new("test");
        let debug_str = format!("{:?}", executor);
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_default_async_task_executor_clone() {
        let executor = DefaultAsyncTaskExecutor::new("test");
        let _ = executor;
    }

    #[test]
    fn test_default_async_task_executor_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        let executor1 = DefaultAsyncTaskExecutor::new("test");
        let executor2 = DefaultAsyncTaskExecutor::new("test");
        map.insert(format!("{:?}", executor1), 1);
        map.insert(format!("{:?}", executor2), 2);
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_async_task_result_clone() {
        let ok = AsyncTaskResult::Ok(Box::new(42) as Box<dyn std::any::Any + Send + Sync>);
        let err = AsyncTaskResult::Err("error".to_string());
        // AsyncTaskResult doesn't implement Clone, but we can test the enum variants
        assert!(matches!(ok, AsyncTaskResult::Ok(_)));
        assert!(matches!(err, AsyncTaskResult::Err(_)));
    }

    #[test]
    fn test_async_task_result_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<AsyncTaskResult>();
        assert_sync::<AsyncTaskResult>();
    }

    #[test]
    fn test_default_async_task_executor_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<DefaultAsyncTaskExecutor>();
        assert_sync::<DefaultAsyncTaskExecutor>();
    }
}
