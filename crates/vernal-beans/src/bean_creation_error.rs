//! BeanCreationError — 对应 Spring `org.springframework.beans.BeanCreationError`。
//!
//! Bean 创建过程中发生的错误，区别于 `factory::BeanCreationException`。
//! 此类型位于 beans 包根级别，用于表示 Bean 创建的基本错误，
//! 而 `factory::BeanCreationException` 包含更丰富的上下文信息（如 Bean 名称、资源描述等）。
//!
//! # Spring 对标
//!
//! Java Spring 中 `org.springframework.beans.factory.BeanCreationException` 的简化版本，
//! 用于 beans 核心层面的错误传播。

use std::fmt;

/// Bean 创建错误。
///
/// 对应 Spring `org.springframework.beans.BeanCreationError`。
///
/// 当 Bean 创建过程中发生基本错误时抛出。
/// 与 `factory::BeanCreationException` 不同，此类型侧重于核心 beans 层面的错误，
/// 不包含 Bean 名称和资源描述等工厂级上下文。
#[derive(Debug)]
pub struct BeanCreationError {
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl BeanCreationError {
    /// 创建一个新的 BeanCreationError。
    ///
    /// # Arguments
    ///
    /// * `message` - 错误消息
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// 创建一个新的 BeanCreationError，并附带根原因。
    ///
    /// # Arguments
    ///
    /// * `message` - 错误消息
    /// * `cause` - 根原因
    pub fn with_cause(
        message: impl Into<String>,
        cause: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(cause)),
        }
    }

    /// 获取错误消息。
    pub fn message(&self) -> &str {
        &self.message
    }

    /// 获取根原因（如果有）。
    pub fn cause(&self) -> Option<&(dyn std::error::Error + Send + Sync + 'static)> {
        self.source.as_ref().map(|e| e.as_ref())
    }
}

impl fmt::Display for BeanCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BeanCreationError: {}", self.message)
    }
}

impl std::error::Error for BeanCreationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let e = BeanCreationError::new("test error");
        assert_eq!(e.message(), "test error");
        assert!(e.cause().is_none());
    }

    #[test]
    fn test_with_cause() {
        let cause = std::io::Error::new(std::io::ErrorKind::Other, "inner error");
        let e = BeanCreationError::with_cause("outer error", cause);
        assert_eq!(e.message(), "outer error");
        assert!(e.cause().is_some());
    }

    #[test]
    fn test_display() {
        let e = BeanCreationError::new("something went wrong");
        assert_eq!(
            format!("{}", e),
            "BeanCreationError: something went wrong"
        );
    }

    #[test]
    fn test_error_trait() {
        let cause = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let e = BeanCreationError::with_cause("wrapper", cause);
        let error: &dyn std::error::Error = &e;
        assert!(error.source().is_some());
    }

    #[test]
    fn test_error_trait_no_source() {
        let e = BeanCreationError::new("no cause");
        let error: &dyn std::error::Error = &e;
        assert!(error.source().is_none());
    }
}
