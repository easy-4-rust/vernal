//! BeanDefinitionValidationException — Bean 定义验证异常。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionValidationException`。
//!
//! 在 Spring 中，当 Bean 定义验证失败时（如配置不合法、
//! 必要属性缺失等）抛出此异常。

use std::fmt;

/// Bean 定义验证异常。
///
/// 对应 Spring 的 `BeanDefinitionValidationException`。
///
/// 当 Bean 定义不合法时抛出，例如：
/// - 工厂方法与构造器冲突
/// - Scope 配置错误
/// - 必要属性缺失
#[derive(Debug, Clone)]
pub struct BeanDefinitionValidationException {
    bean_name: String,
    message: String,
}

impl BeanDefinitionValidationException {
    /// 创建新的异常。
    pub fn new(message: impl Into<String>) -> Self {
        Self { bean_name: String::new(), message: message.into() }
    }

    /// 创建带 Bean 名称的异常。
    pub fn with_bean_name(bean_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self { bean_name: bean_name.into(), message: message.into() }
    }

    /// 获取错误消息。
    pub fn message(&self) -> &str { &self.message }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> &str { &self.bean_name }
}

impl fmt::Display for BeanDefinitionValidationException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.bean_name.is_empty() {
            write!(f, "{}", self.message)
        } else {
            write!(f, "Bean '{}' validation failed: {}", self.bean_name, self.message)
        }
    }
}

impl std::error::Error for BeanDefinitionValidationException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_exception_has_message() {
        let e = BeanDefinitionValidationException::new("invalid config");
        assert_eq!(e.message(), "invalid config");
    }

    #[test]
    fn with_bean_name() {
        let e = BeanDefinitionValidationException::with_bean_name("dataSource", "missing url");
        assert_eq!(e.bean_name(), "dataSource");
        assert_eq!(e.message(), "missing url");
    }

    #[test]
    fn display_with_bean_name() {
        let e = BeanDefinitionValidationException::with_bean_name("myBean", "bad scope");
        assert_eq!(format!("{}", e), "Bean 'myBean' validation failed: bad scope");
    }

    #[test]
    fn display_without_bean_name() {
        let e = BeanDefinitionValidationException::new("generic error");
        assert_eq!(format!("{}", e), "generic error");
    }
}
