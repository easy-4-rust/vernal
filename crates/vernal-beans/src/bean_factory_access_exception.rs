//! BeanFactoryAccessException — Spring 风格 Bean 工厂访问异常。
//!
//! 对应 Java 类：`org.springframework.beans.BeansException` 的子类。
//!
//! 在 Spring 中，当 Bean 工厂操作失败时（如访问被拒绝、Bean 不存在等），
//! 抛出此异常。它是 `BeansException` 的具体子类。

use std::fmt;

/// Bean 工厂访问异常。
///
/// 对应 Spring 的 `BeanFactoryAccessException`（`BeansException` 子类）。
///
/// 当 Bean 工厂操作失败时抛出，例如：
/// - 访问被拒绝
/// - Bean 不存在
/// - 类型转换失败
/// - 循环依赖检测
#[derive(Debug)]
pub struct BeanFactoryAccessException {
    /// 错误消息
    message: String,
    /// 异常根原因
    cause: Option<Box<dyn std::error::Error + Send + Sync>>,
    /// 相关的 Bean 名称（如果有）
    bean_name: Option<String>,
}

impl BeanFactoryAccessException {
    /// 创建新的 BeanFactoryAccessException。
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            cause: None,
            bean_name: None,
        }
    }

    /// 创建带 Bean 名称的异常。
    pub fn with_bean_name(message: impl Into<String>, bean_name: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            cause: None,
            bean_name: Some(bean_name.into()),
        }
    }

    /// 创建带根原因的异常。
    pub fn with_cause(
        message: impl Into<String>,
        cause: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            cause: Some(Box::new(cause)),
            bean_name: None,
        }
    }

    /// 创建带 Bean 名称和根原因的异常。
    pub fn with_bean_name_and_cause(
        message: impl Into<String>,
        bean_name: impl Into<String>,
        cause: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            cause: Some(Box::new(cause)),
            bean_name: Some(bean_name.into()),
        }
    }

    /// 获取异常消息。
    pub fn message(&self) -> &str {
        &self.message
    }

    /// 获取根原因（如果有）。
    pub fn source_error(&self) -> Option<&(dyn std::error::Error + Send + Sync + 'static)> {
        self.cause.as_ref().map(|e| e.as_ref())
    }

    /// 获取相关的 Bean 名称（如果有）。
    pub fn bean_name(&self) -> Option<&str> {
        self.bean_name.as_deref()
    }

    /// 是否与特定 Bean 相关。
    pub fn has_bean_name(&self) -> bool {
        self.bean_name.is_some()
    }
}

impl fmt::Display for BeanFactoryAccessException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.bean_name {
            Some(name) => write!(
                f,
                "BeanFactoryAccessException for bean '{}': {}",
                name, self.message
            ),
            None => write!(f, "BeanFactoryAccessException: {}", self.message),
        }
    }
}

impl std::error::Error for BeanFactoryAccessException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_exception_has_message() {
        let e = BeanFactoryAccessException::new("bean not found");
        assert_eq!(e.message(), "bean not found");
        assert!(!e.has_bean_name());
        assert!(e.bean_name().is_none());
    }

    #[test]
    fn with_bean_name() {
        let e = BeanFactoryAccessException::with_bean_name("creation failed", "myService");
        assert_eq!(e.message(), "creation failed");
        assert!(e.has_bean_name());
        assert_eq!(e.bean_name(), Some("myService"));
    }

    #[test]
    fn with_cause() {
        let cause = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let e = BeanFactoryAccessException::with_cause("wrapper error", cause);
        assert!(e.source_error().is_some());
    }

    #[test]
    fn display_format_with_bean_name() {
        let e = BeanFactoryAccessException::with_bean_name("timeout", "dataSource");
        assert_eq!(
            format!("{}", e),
            "BeanFactoryAccessException for bean 'dataSource': timeout"
        );
    }

    #[test]
    fn display_format_without_bean_name() {
        let e = BeanFactoryAccessException::new("generic error");
        assert_eq!(
            format!("{}", e),
            "BeanFactoryAccessException: generic error"
        );
    }

    #[test]
    fn error_trait_source() {
        let cause = std::io::Error::new(std::io::ErrorKind::Other, "inner");
        let e = BeanFactoryAccessException::with_cause("outer", cause);
        let err: &dyn std::error::Error = &e;
        assert!(err.source().is_some());
    }

    #[test]
    fn with_bean_name_and_cause() {
        let cause = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let e = BeanFactoryAccessException::with_bean_name_and_cause(
            "failed to create",
            "myService",
            cause,
        );
        assert_eq!(e.message(), "failed to create");
        assert!(e.has_bean_name());
        assert_eq!(e.bean_name(), Some("myService"));
        assert!(e.source_error().is_some());
    }

    #[test]
    fn with_bean_name_and_cause_display() {
        let cause = std::io::Error::new(std::io::ErrorKind::Other, "root");
        let e = BeanFactoryAccessException::with_bean_name_and_cause(
            "init failed",
            "dataSource",
            cause,
        );
        let display = format!("{}", e);
        assert!(display.contains("dataSource"));
        assert!(display.contains("init failed"));
    }

    #[test]
    fn error_trait_no_source() {
        let e = BeanFactoryAccessException::new("no cause");
        let err: &dyn std::error::Error = &e;
        assert!(err.source().is_none());
    }

    #[test]
    fn new_with_string_owned() {
        let msg = String::from("owned message");
        let e = BeanFactoryAccessException::new(msg);
        assert_eq!(e.message(), "owned message");
    }

    #[test]
    fn with_bean_name_str_refs() {
        let e = BeanFactoryAccessException::with_bean_name("msg", "bean");
        assert_eq!(e.message(), "msg");
        assert_eq!(e.bean_name(), Some("bean"));
    }

    #[test]
    fn display_trait_implementation() {
        let e = BeanFactoryAccessException::new("test");
        let formatted = format!("{e}");
        assert!(formatted.contains("BeanFactoryAccessException"));
        assert!(formatted.contains("test"));
    }

    #[test]
    fn error_trait_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<BeanFactoryAccessException>();
    }
}
