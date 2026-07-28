//! 对标 `org.springframework.scheduling.aspectj` 包。
//!
//! 提供 AspectJ 异步执行切面的 Rust 等价实现，覆盖：
//! - `AbstractAsyncExecutionAspect`：异步切面抽象层
//! - `AnnotationAsyncExecutionAspect`：`@Async` 注解驱动
//! - `AspectJAsyncConfiguration`：Spring @Configuration 等价
//! - `AsyncTaskExecutor`：异步任务执行器 trait
//! - `AsyncUncaughtExceptionHandler`：异步异常处理器 trait

mod async_task_executor;
mod async_uncaught_exception_handler;
mod abstract_async_execution_aspect;
mod annotation_async_execution_aspect;
mod aspectj_async_configuration;

pub use async_task_executor::AsyncTaskExecutor;
pub use async_uncaught_exception_handler::AsyncUncaughtExceptionHandler;
pub use abstract_async_execution_aspect::AbstractAsyncExecutionAspect;
pub use annotation_async_execution_aspect::AnnotationAsyncExecutionAspect;
pub use aspectj_async_configuration::AspectJAsyncConfiguration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduling_aspectj() {
        let aspect = AbstractAsyncExecutionAspect::new();
        let _ = aspect;
    }

    #[test]
    fn test_annotation_async_execution_aspect() {
        let aspect = AnnotationAsyncExecutionAspect::new();
        let _ = aspect;
    }

    #[test]
    fn test_aspectj_async_configuration() {
        let config = AspectJAsyncConfiguration::new();
        let _ = config;
    }
}
