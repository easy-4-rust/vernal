//! 对应 aspect-rs：aspect-core/src/error.rs::AspectError
//! 语义参照 spring-aop：AopInvocationException
//!
//! 切面执行错误对象（三变体）。
//! 移植自 aspect-rs 的 AspectError，与现有 InvocationError 共存。

use std::{error::Error, fmt};

/// 切面执行过程中可观察的结构化失败。
///
/// 对应 aspect-rs 的 `AspectError`，保留三个变体语义。
#[derive(Debug)]
pub enum AspectError {
    /// 目标函数执行失败。
    ExecutionError {
        /// 错误描述。
        message: String,
        /// 可选的底层错误。
        source: Option<Box<dyn Error + Send + Sync>>,
    },

    /// 过程宏织入阶段失败（spring-aop 无对偶，Rust 特有）。
    WeavingError {
        /// 织入错误描述。
        message: String,
    },

    /// 用户定义的自定义错误。
    Custom(Box<dyn Error + Send + Sync>),
}

impl AspectError {
    /// 创建执行错误。
    #[must_use]
    pub fn execution(message: impl Into<String>) -> Self {
        Self::ExecutionError {
            message: message.into(),
            source: None,
        }
    }

    /// 创建带底层错误的执行错误。
    #[must_use]
    pub fn execution_with_source(
        message: impl Into<String>,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self::ExecutionError {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// 创建织入错误。
    #[must_use]
    pub fn weaving(message: impl Into<String>) -> Self {
        Self::WeavingError {
            message: message.into(),
        }
    }

    /// 从任意错误类型创建自定义错误。
    #[must_use]
    pub fn custom(error: impl Error + Send + Sync + 'static) -> Self {
        Self::Custom(Box::new(error))
    }
}

impl fmt::Display for AspectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExecutionError { message, .. } => {
                write!(formatter, "Execution error: {message}")
            }
            Self::WeavingError { message } => {
                write!(formatter, "Weaving error: {message}")
            }
            Self::Custom(error) => write!(formatter, "Custom error: {error}"),
        }
    }
}

impl Error for AspectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ExecutionError { source, .. } => source
                .as_ref()
                .map(|error| error.as_ref() as &(dyn Error + 'static)),
            Self::Custom(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl From<String> for AspectError {
    fn from(message: String) -> Self {
        Self::execution(message)
    }
}

impl From<&str> for AspectError {
    fn from(message: &str) -> Self {
        Self::execution(message)
    }
}

impl From<Box<dyn Error + Send + Sync>> for AspectError {
    fn from(error: Box<dyn Error + Send + Sync>) -> Self {
        Self::Custom(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn execution_error_display() {
        let error = AspectError::execution("test error");
        assert!(matches!(error, AspectError::ExecutionError { .. }));
        assert_eq!(error.to_string(), "Execution error: test error");
    }

    #[test]
    fn execution_error_with_source_has_cause() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let error = AspectError::execution_with_source("read failed", io_error);
        assert!(error.source().is_some());
    }

    #[test]
    fn weaving_error_display() {
        let error = AspectError::weaving("invalid pointcut");
        assert!(matches!(error, AspectError::WeavingError { .. }));
        assert_eq!(error.to_string(), "Weaving error: invalid pointcut");
    }

    #[test]
    fn custom_error_display() {
        let io_error = io::Error::new(io::ErrorKind::Other, "custom");
        let error = AspectError::custom(io_error);
        assert!(matches!(error, AspectError::Custom(_)));
    }

    #[test]
    fn from_string_conversion() {
        let error: AspectError = "error message".into();
        assert!(matches!(error, AspectError::ExecutionError { .. }));
    }

    #[test]
    fn from_str_conversion() {
        let error: AspectError = "error message".into();
        assert!(matches!(error, AspectError::ExecutionError { .. }));
    }

    #[test]
    fn from_boxed_error_conversion() {
        let io_error: Box<dyn Error + Send + Sync> =
            Box::new(io::Error::new(io::ErrorKind::Other, "boxed"));
        let error: AspectError = io_error.into();
        assert!(matches!(error, AspectError::Custom(_)));
    }
}
