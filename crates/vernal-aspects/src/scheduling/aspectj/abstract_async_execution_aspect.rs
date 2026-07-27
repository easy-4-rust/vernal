//! 对标 `org.springframework.scheduling.aspectj.AbstractAsyncExecutionAspect` 抽象 Aspect。
//!
//! 异步切面抽象层：封装异步方法的执行逻辑。

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use super::async_task_executor::{AsyncTaskExecutor, AsyncTaskResult, DefaultAsyncTaskExecutor};
use super::async_uncaught_exception_handler::{
    AsyncUncaughtExceptionHandler, DefaultAsyncUncaughtExceptionHandler,
};

/// 方法元数据（异步模块用）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MethodMetadata {
    /// 方法所属类型的完全限定名。
    pub type_name: &'static str,
    /// 方法名。
    pub method_name: &'static str,
    /// 方法返回类型名。
    pub return_type: &'static str,
}

impl MethodMetadata {
    /// 创建新的方法元数据。
    pub fn new(type_name: &'static str, method_name: &'static str, return_type: &'static str) -> Self {
        Self {
            type_name,
            method_name,
            return_type,
        }
    }

    /// 返回方法的完全限定签名。
    pub fn qualified_name(&self) -> String {
        format!("{}#{}", self.type_name, self.method_name)
    }
}

/// 异步执行结果。
pub enum AsyncExecutionResult {
    /// 方法返回 void（异步 fire-and-forget）。
    Void,
    /// 方法返回 Future。
    Future(Pin<Box<dyn Future<Output = AsyncTaskResult> + Send>>),
}

/// 异步切面抽象基类。
///
/// 对标 Spring 的 `AbstractAsyncExecutionAspect`。
/// 封装了异步方法的执行逻辑。
pub struct AbstractAsyncExecutionAspect {
    /// 异步任务执行器。
    executor: Option<Arc<dyn AsyncTaskExecutor>>,
    /// 异步异常处理器。
    exception_handler: Option<Arc<dyn AsyncUncaughtExceptionHandler>>,
    /// 默认执行器名称。
    default_executor_name: Option<String>,
}

impl AbstractAsyncExecutionAspect {
    /// 创建异步切面抽象实例。
    pub fn new() -> Self {
        Self {
            executor: None,
            exception_handler: None,
            default_executor_name: None,
        }
    }

    /// 设置执行器。
    pub fn set_executor(&mut self, executor: Arc<dyn AsyncTaskExecutor>) {
        self.executor = Some(executor);
    }

    /// 设置异常处理器。
    pub fn set_exception_handler(&mut self, handler: Arc<dyn AsyncUncaughtExceptionHandler>) {
        self.exception_handler = Some(handler);
    }

    /// 设置默认执行器名称。
    pub fn set_default_executor_name(&mut self, name: String) {
        self.default_executor_name = Some(name);
    }

    /// 根据方法确定异步执行器。
    ///
    /// 对标 Spring 的 `determineAsyncExecutor(Method method)` 方法。
    pub fn determine_async_executor(&self, _method: &MethodMetadata) -> Option<Arc<dyn AsyncTaskExecutor>> {
        self.executor.clone()
    }

    /// 执行异步方法。
    ///
    /// 对标 Spring 的 `AbstractAsyncExecutionAspect#around` advice。
    ///
    /// # 执行流程
    ///
    /// 1. 确定执行器
    /// 2. 如果没有执行器，同步执行
    /// 3. 如果有执行器，异步执行
    pub fn execute_async<F>(
        &self,
        method: &MethodMetadata,
        callback: F,
    ) -> AsyncExecutionResult
    where
        F: Future<Output = AsyncTaskResult> + Send + 'static,
    {
        match self.determine_async_executor(method) {
            Some(executor) => {
                // 异步执行
                let task = Box::pin(callback);
                AsyncExecutionResult::Future(executor.submit(task))
            }
            None => {
                // 同步执行（回退）
                let task = Box::pin(callback);
                AsyncExecutionResult::Future(task)
            }
        }
    }

    /// 处理异步异常。
    pub fn handle_error(
        &self,
        exception: &dyn std::any::Any,
        method: &MethodMetadata,
        args: &[Box<dyn std::any::Any + Send + Sync>],
    ) {
        if let Some(handler) = &self.exception_handler {
            handler.handle_uncaught_exception(exception, &method.method_name, args);
        }
    }
}

impl Default for AbstractAsyncExecutionAspect {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abstract_async_execution_aspect_creation() {
        let aspect = AbstractAsyncExecutionAspect::new();
        assert!(aspect.executor.is_none());
        assert!(aspect.exception_handler.is_none());
    }

    #[test]
    fn test_determine_async_executor_no_executor() {
        let aspect = AbstractAsyncExecutionAspect::new();
        let method = MethodMetadata::new("Foo", "bar", "void");
        assert!(aspect.determine_async_executor(&method).is_none());
    }

    #[test]
    fn test_determine_async_executor_with_executor() {
        let mut aspect = AbstractAsyncExecutionAspect::new();
        let executor = Arc::new(DefaultAsyncTaskExecutor::new("test"));
        aspect.set_executor(executor.clone());
        let method = MethodMetadata::new("Foo", "bar", "void");
        let found = aspect.determine_async_executor(&method);
        assert!(found.is_some());
        assert_eq!(found.unwrap().get_executor_name(), "test");
    }

    #[test]
    fn test_execute_async_no_executor() {
        let aspect = AbstractAsyncExecutionAspect::new();
        let method = MethodMetadata::new("Foo", "bar", "void");

        let result = aspect.execute_async(&method, async {
            AsyncTaskResult::Ok(Box::new(42) as Box<dyn std::any::Any + Send + Sync>)
        });

        match result {
            AsyncExecutionResult::Future(_) => {} // OK
            _ => panic!("Expected Future result"),
        }
    }

    #[test]
    fn test_handle_error_no_handler() {
        let aspect = AbstractAsyncExecutionAspect::new();
        let method = MethodMetadata::new("Foo", "bar", "void");
        // 不应 panic
        aspect.handle_error(&"test error", &method, &[]);
    }

    #[test]
    fn test_abstract_async_execution_aspect_default() {
        let aspect = AbstractAsyncExecutionAspect::default();
        assert!(aspect.executor.is_none());
    }
}
