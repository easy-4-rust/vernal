//! BeanDefinitionOverrideException — Bean 定义覆盖异常。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionOverrideException`。
//!
//! 在 Spring 中，当注册的 Bean 定义名称与已有定义冲突，
//! 且 `allowBeanDefinitionOverriding` 为 `false` 时抛出此异常。
//! 这是防止意外覆盖的安全机制。

use std::fmt;

/// Bean 定义覆盖异常。
///
/// 对应 Spring 的 `BeanDefinitionOverrideException`。
///
/// 当尝试注册已存在的 Bean 定义名称时抛出。
#[derive(Debug, Clone)]
pub struct BeanDefinitionOverrideException {
    /// Bean 名称
    bean_name: String,
    /// 错误消息
    message: String,
}

impl BeanDefinitionOverrideException {
    /// 创建新的异常。
    pub fn new(message: impl Into<String>) -> Self {
        Self { bean_name: String::new(), message: message.into() }
    }

    /// 创建带 Bean 名称的异常。
    pub fn with_bean_name(
        bean_name: impl Into<String>,
        existing_name: impl Into<String>,
    ) -> Self {
        let bean = bean_name.into();
        let existing = existing_name.into();
        Self {
            bean_name: bean.clone(),
            message: format!(
                "Cannot register bean definition '{}' for bean '{}': there is already [{}] bound",
                bean, bean, existing
            ),
        }
    }

    /// 获取错误消息。
    pub fn message(&self) -> &str { &self.message }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> &str { &self.bean_name }
}

impl fmt::Display for BeanDefinitionOverrideException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BeanDefinitionOverrideException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_exception_has_message() {
        let e = BeanDefinitionOverrideException::new("override not allowed");
        assert_eq!(e.message(), "override not allowed");
    }

    #[test]
    fn with_bean_name() {
        let e = BeanDefinitionOverrideException::with_bean_name("myService", "existingDef");
        assert_eq!(e.bean_name(), "myService");
        assert!(e.message().contains("myService"));
    }

    #[test]
    fn display_and_error_trait() {
        let e = BeanDefinitionOverrideException::new("test error");
        assert_eq!(format!("{}", e), "test error");
        let err: &dyn std::error::Error = &e;
        assert!(!err.to_string().is_empty());
    }
}
