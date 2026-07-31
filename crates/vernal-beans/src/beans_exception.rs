//! BeansException — 对应 Spring `org.springframework.beans.BeansException`。
//!
//! beans 包及其子包中所有异常的抽象基类。
//! 这是一个运行时（非受检）异常。Beans 异常通常是致命的。

use std::fmt;

/// beans 包及其子包中所有异常的抽象基类。
///
/// 对应 Java 类：`org.springframework.beans.BeansException`。
///
/// 这是一个运行时（非受检）异常。Beans 异常通常是致命的，
/// 没有理由让它们成为受检异常。
#[derive(Debug)]
pub struct BeansException {
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl BeansException {
    /// 创建一个新的 BeansException，包含指定的错误消息。
    ///
    /// 对应 Java 构造器：`BeansException(String msg)`
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// 创建一个新的 BeansException，包含指定的错误消息和根原因。
    ///
    /// 对应 Java 构造器：`BeansException(String msg, Throwable cause)`
    pub fn with_cause(message: impl Into<String>, cause: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(cause)),
        }
    }

    /// 获取异常的详细消息。
    pub fn message(&self) -> &str {
        &self.message
    }

    /// 获取异常的根原因（如果有）。
    pub fn source(&self) -> Option<&(dyn std::error::Error + Send + Sync + 'static)> {
        self.source.as_ref().map(|e| e.as_ref())
    }
}

impl fmt::Display for BeansException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BeansException: {}", self.message)
    }
}

impl std::error::Error for BeansException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beans_exception_new() {
        let e = BeansException::new("test error");
        assert_eq!(e.message(), "test error");
        assert!(e.source().is_none());
    }

    #[test]
    fn test_beans_exception_with_cause() {
        let cause = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let e = BeansException::with_cause("wrapper error", cause);
        assert_eq!(e.message(), "wrapper error");
        assert!(e.source().is_some());
    }

    #[test]
    fn test_beans_exception_display() {
        let e = BeansException::new("test");
        assert_eq!(format!("{}", e), "BeansException: test");
    }

    #[test]
    fn test_beans_exception_error_trait() {
        let e = BeansException::new("test");
        let error: &dyn std::error::Error = &e;
        assert!(error.source().is_none());
    }
}
