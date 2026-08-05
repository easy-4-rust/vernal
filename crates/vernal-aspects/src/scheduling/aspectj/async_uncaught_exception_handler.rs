//! 对标 `org.springframework.aop.interceptor.AsyncUncaughtExceptionHandler` 接口。
//!
//! 异步异常处理器：处理异步方法执行中的异常。

use std::any::Any;

/// 异步异常处理器 trait。
///
/// 对标 Spring 的 `AsyncUncaughtExceptionHandler` 接口。
/// 处理异步方法执行中未捕获的异常。
pub trait AsyncUncaughtExceptionHandler: Send + Sync + 'static {
    /// 处理异常。
    ///
    /// 对应 Spring 的 `AsyncUncaughtExceptionHandler#handleUncaughtException(Throwable, Method, Object...)`。
    ///
    /// # Arguments
    ///
    /// * `exception` - 异常信息
    /// * `method_name` - 方法名
    /// * `args` - 方法参数
    fn handle_uncaught_exception(
        &self,
        exception: &dyn Any,
        method_name: &str,
        args: &[Box<dyn Any + Send + Sync>],
    );
}

/// 默认的异步异常处理器（打印异常信息）。
#[allow(dead_code)] // Java 镜像脚手架：当前阶段未在切面中实际构造，供测试使用
pub struct DefaultAsyncUncaughtExceptionHandler;

impl AsyncUncaughtExceptionHandler for DefaultAsyncUncaughtExceptionHandler {
    fn handle_uncaught_exception(
        &self,
        exception: &dyn Any,
        method_name: &str,
        _args: &[Box<dyn Any + Send + Sync>],
    ) {
        if let Some(msg) = exception.downcast_ref::<&str>() {
            eprintln!(
                "[AsyncUncaughtExceptionHandler] Method '{}' threw exception: {}",
                method_name, msg
            );
        } else if let Some(msg) = exception.downcast_ref::<String>() {
            eprintln!(
                "[AsyncUncaughtExceptionHandler] Method '{}' threw exception: {}",
                method_name, msg
            );
        } else {
            eprintln!(
                "[AsyncUncaughtExceptionHandler] Method '{}' threw an unknown exception",
                method_name
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_handler_str() {
        let handler = DefaultAsyncUncaughtExceptionHandler;
        handler.handle_uncaught_exception(&"test error", "testMethod", &[]);
    }

    #[test]
    fn test_default_handler_string() {
        let handler = DefaultAsyncUncaughtExceptionHandler;
        let msg = String::from("string error");
        handler.handle_uncaught_exception(&msg, "testMethod", &[]);
    }

    #[test]
    fn test_default_handler_unknown() {
        let handler = DefaultAsyncUncaughtExceptionHandler;
        handler.handle_uncaught_exception(&42, "testMethod", &[]);
    }

    #[test]
    fn test_default_handler_with_args() {
        let handler = DefaultAsyncUncaughtExceptionHandler;
        let args: Vec<Box<dyn std::any::Any + Send + Sync>> =
            vec![Box::new(42), Box::new("test".to_string())];
        handler.handle_uncaught_exception(&"error", "testMethod", &args);
    }

    #[test]
    fn test_async_uncaught_exception_handler_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<DefaultAsyncUncaughtExceptionHandler>();
        assert_sync::<DefaultAsyncUncaughtExceptionHandler>();
    }
}
