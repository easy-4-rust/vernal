//! ImplicitlyAppearedSingletonException — Spring 风格的隐式单例异常。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.ImplicitlyAppearedSingletonException`。
//!
//! 在 Spring 中，当一个原本不是单例的 Bean 被隐式地作为单例处理时抛出此异常。
//! 这通常发生在 `SmartInitializingSingleton` 回调期间，
//! 某些 Bean 依赖了未在作用域中注册的单例 Bean。
//!
//! ## 设计说明
//!
//! 在 vernal 中，此异常用于标识作用域边界违规的场景。

use std::fmt;

/// 隐式单例异常。
///
/// 对应 Spring 的 `ImplicitlyAppearedSingletonException`。
///
/// 当 Bean 的作用域在运行时被隐式改变（例如从 prototype 变为 singleton）时抛出。
#[derive(Debug)]
pub struct ImplicitlyAppearedSingletonException {
    /// Bean 名称。
    bean_name: String,
    /// 异常描述消息。
    message: String,
}

impl ImplicitlyAppearedSingletonException {
    /// 创建隐式单例异常。
    ///
    /// # 参数
    /// - `bean_name` — 涉及的 Bean 名称
    pub fn new(bean_name: impl Into<String>) -> Self {
        let bean_name = bean_name.into();
        let message = format!(
            "Bean '{}' implicitly appeared as singleton; \
             this typically happens when a non-singleton bean \
             is required as a dependency of a singleton bean",
            bean_name
        );
        Self { bean_name, message }
    }

    /// 创建带自定义消息的异常。
    pub fn with_message(bean_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            bean_name: bean_name.into(),
            message: message.into(),
        }
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }

    /// 获取异常消息。
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ImplicitlyAppearedSingletonException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ImplicitlyAppearedSingletonException: {}", self.message)
    }
}

impl std::error::Error for ImplicitlyAppearedSingletonException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exception_contains_bean_name() {
        let exc = ImplicitlyAppearedSingletonException::new("myService");
        assert_eq!(exc.bean_name(), "myService");
        assert!(exc.message().contains("myService"));
    }

    #[test]
    fn exception_display_format() {
        let exc = ImplicitlyAppearedSingletonException::new("testBean");
        let display = format!("{}", exc);
        assert!(display.starts_with("ImplicitlyAppearedSingletonException:"));
        assert!(display.contains("testBean"));
    }

    #[test]
    fn exception_with_custom_message() {
        let exc = ImplicitlyAppearedSingletonException::with_message("bean1", "custom error");
        assert_eq!(exc.bean_name(), "bean1");
        assert_eq!(exc.message(), "custom error");
    }

    #[test]
    fn exception_is_std_error() {
        let exc = ImplicitlyAppearedSingletonException::new("bean");
        let err: &dyn std::error::Error = &exc;
        assert!(!err.to_string().is_empty());
    }
}
