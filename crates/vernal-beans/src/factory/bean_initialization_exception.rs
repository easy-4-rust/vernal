//! BeanInitializationException — 对应 Spring `org.springframework.beans.factory.BeanInitializationException`。
//!
//! Bean 初始化失败时抛出的异常。

use std::fmt;

/// Bean 初始化失败时抛出的异常。
///
/// 对应 Java 类：`org.springframework.beans.factory.BeanInitializationException`。
#[derive(Debug)]
pub struct BeanInitializationException {
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl BeanInitializationException {
    /// 创建一个新的 BeanInitializationException。
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// 创建一个带有原因的 BeanInitializationException。
    pub fn with_cause(
        message: impl Into<String>,
        cause: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(cause)),
        }
    }

    /// 获取异常消息。
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for BeanInitializationException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to initialize bean: {}", self.message)
    }
}

impl std::error::Error for BeanInitializationException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_new() {
        let e = BeanInitializationException::new("init failed");
        assert_eq!(e.message(), "init failed");
        assert!(e.source().is_none());
    }

    #[test]
    fn test_with_cause() {
        let cause = std::io::Error::new(std::io::ErrorKind::Other, "inner");
        let e = BeanInitializationException::with_cause("outer", cause);
        assert!(e.source().is_some());
    }

    #[test]
    fn test_display() {
        let e = BeanInitializationException::new("test");
        assert!(format!("{}", e).contains("test"));
    }
}
