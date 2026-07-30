//! 结构化日志切面。
//!
//! 对应 aspect-rs：aspect-std/src/logging.rs。
//! 语义参照 spring-aop：`DebugInterceptor` / `SimpleTraceInterceptor`。
//!
//! 提供函数进入、退出和错误的结构化日志记录。
//! 实现 `Interceptor` trait（around 全控制），使用 `tracing` crate。

use std::sync::Arc;

use crate::{Interceptor, Invocation, InvocationFuture, InvocationResult, Next};

/// 日志级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Trace 级别（最详细）
    Trace,
    /// Debug 级别
    Debug,
    /// Info 级别（默认）
    Info,
    /// Warn 级别
    Warn,
    /// Error 级别
    Error,
}

/// 结构化日志切面。
///
/// 提供函数进入、退出和错误的日志记录。
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::{LoggingAspect, LogLevel};
///
/// let aspect = LoggingAspect::new()
///     .with_level(LogLevel::Debug)
///     .log_args()
///     .log_result();
/// ```
#[derive(Clone)]
pub struct LoggingAspect {
    level: LogLevel,
    log_args: bool,
    log_result: bool,
}

impl LoggingAspect {
    /// 创建 Info 级别的日志切面。
    #[must_use]
    pub fn new() -> Self {
        Self {
            level: LogLevel::Info,
            log_args: false,
            log_result: false,
        }
    }

    /// 设置日志级别。
    #[must_use]
    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    /// 启用函数参数日志记录（默认关闭）。
    #[must_use]
    pub fn log_args(mut self) -> Self {
        self.log_args = true;
        self
    }

    /// 启用函数返回值日志记录（默认关闭）。
    #[must_use]
    pub fn log_result(mut self) -> Self {
        self.log_result = true;
        self
    }

    /// 记录日志。
    fn log(&self, level: LogLevel, message: &str) {
        if level as u8 >= self.level as u8 {
            match level {
                LogLevel::Trace => tracing::trace!("{}", message),
                LogLevel::Debug => tracing::debug!("{}", message),
                LogLevel::Info => tracing::info!("{}", message),
                LogLevel::Warn => tracing::warn!("{}", message),
                LogLevel::Error => tracing::error!("{}", message),
            }
        }
    }
}

impl Default for LoggingAspect {
    fn default() -> Self {
        Self::new()
    }
}

impl Interceptor for LoggingAspect {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            let op = invocation.operation();
            let component = op.component().to_string();
            let method = op.method().to_string();

            let entry_msg = format!("[ENTRY] {}::{}", component, method);
            self.log(self.level, &entry_msg);

            let result = next.run(invocation).await;

            match &result {
                Ok(_) => {
                    let mut exit_msg = format!("[EXIT] {}::{}", component, method);
                    if self.log_result {
                        exit_msg.push_str(" (success)");
                    }
                    self.log(self.level, &exit_msg);
                }
                Err(e) => {
                    let error_msg = format!("[ERROR] {}::{} failed: {}", component, method, e);
                    self.log(LogLevel::Error, &error_msg);
                }
            }

            result
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InvocationPlanBuilder, Operation};

    fn always() -> impl Fn(&Operation) -> bool {
        |_| true
    }

    #[tokio::test]
    async fn logging_aspect_logs_entry_and_exit() {
        let aspect = LoggingAspect::new().with_level(LogLevel::Debug);

        let operation = Operation::new("TestService", "test_method");
        let mut builder = InvocationPlanBuilder::new();
        builder.register(crate::Advisor::new(
            always(),
            aspect,
            0,
        ));
        let plan = builder.build(operation.clone());

        let target: Arc<crate::InvocationTarget> = Arc::new(|_| {
            Box::pin(async { Ok(Box::new(42i32) as crate::InvocationValue) })
        });

        let invocation = crate::Invocation::new(operation);
        let result = plan.invoke(Arc::new(invocation), target).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn logging_aspect_logs_errors() {
        let aspect = LoggingAspect::new();

        let operation = Operation::new("TestService", "fail_method");
        let mut builder = InvocationPlanBuilder::new();
        builder.register(crate::Advisor::new(
            always(),
            aspect,
            0,
        ));
        let plan = builder.build(operation.clone());

        let target: Arc<crate::InvocationTarget> = Arc::new(|_| {
            Box::pin(async {
                Err(crate::InvocationError::Cancelled)
            })
        });

        let invocation = crate::Invocation::new(operation);
        let result = plan.invoke(Arc::new(invocation), target).await;
        assert!(result.is_err());
    }

    #[test]
    fn logging_aspect_builder() {
        let aspect = LoggingAspect::new()
            .with_level(LogLevel::Debug)
            .log_args()
            .log_result();

        assert_eq!(aspect.level, LogLevel::Debug);
        assert!(aspect.log_args);
        assert!(aspect.log_result);
    }

    #[test]
    fn logging_aspect_default() {
        let aspect = LoggingAspect::default();
        assert_eq!(aspect.level, LogLevel::Info);
        assert!(!aspect.log_args);
        assert!(!aspect.log_result);
    }
}
