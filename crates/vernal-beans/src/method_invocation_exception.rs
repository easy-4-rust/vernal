//! MethodInvocationException — 对应 Java 类：org.springframework.beans.MethodInvocationException。
//!
//! 对应 Spring beans 包。
//!
//! 在 Spring 中，`MethodInvocationException` 是当 Bean 属性的 setter 方法
//! 抛出异常时包装的异常。它将底层的业务异常包装起来，提供 Bean 名称和属性名等上下文信息。
//! 这是一个致命异常，表示 Bean 的属性设置失败。
//!
//! ## 使用场景
//!
//! - 调用 Bean 的 setter 方法时发生异常
//! - 属性注入过程中目标方法抛出异常
//! - 需要保留原始异常链进行调试

use std::fmt;

/// MethodInvocationException — Spring 风格的方法调用异常。
///
/// 对应 Java 类：`org.springframework.beans.MethodInvocationException`。
///
/// 当 Bean 属性 setter 方法抛出异常时，Spring 将其包装为
/// `MethodInvocationException`，保留方法名、Bean 名称和原始异常链。
///
/// ## Java 对比
///
/// | Java | Rust |
/// |------|------|
/// | `MethodInvocationException(String, Method, Throwable)` | `MethodInvocationException::with_cause(msg, method, cause)` |
/// | `getMethod()` | `method_name()` |
/// | `getCause()` | `cause()` |
#[derive(Debug)]
pub struct MethodInvocationException {
    /// 错误消息
    message: String,
    /// 方法名称（setter 或 getter）
    method_name: Option<String>,
    /// Bean 名称
    bean_name: Option<String>,
    /// 属性名称
    property_name: Option<String>,
    /// 原始异常链
    cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl MethodInvocationException {
    /// 创建新的 MethodInvocationException。
    ///
    /// # 参数
    /// - `message` — 错误消息
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            method_name: None,
            bean_name: None,
            property_name: None,
            cause: None,
        }
    }

    /// 创建带原始异常的 MethodInvocationException。
    ///
    /// 对应 Java 构造器：`MethodInvocationException(String msg, Throwable cause)`。
    ///
    /// # 参数
    /// - `message` — 错误消息
    /// - `cause` — 原始异常
    pub fn with_cause(
        message: impl Into<String>,
        cause: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            method_name: None,
            bean_name: None,
            property_name: None,
            cause: Some(Box::new(cause)),
        }
    }

    /// 创建带完整上下文的 MethodInvocationException。
    ///
    /// 对应 Java 构造器：`MethodInvocationException(String msg, Method method, Throwable cause)`。
    ///
    /// # 参数
    /// - `message` — 错误消息
    /// - `method_name` — 方法名称
    /// - `cause` — 原始异常
    pub fn with_method_and_cause(
        message: impl Into<String>,
        method_name: impl Into<String>,
        cause: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            method_name: Some(method_name.into()),
            bean_name: None,
            property_name: None,
            cause: Some(Box::new(cause)),
        }
    }

    /// 设置方法名称。
    pub fn with_method(mut self, method_name: impl Into<String>) -> Self {
        self.method_name = Some(method_name.into());
        self
    }

    /// 设置 Bean 名称。
    pub fn with_bean_name(mut self, bean_name: impl Into<String>) -> Self {
        self.bean_name = Some(bean_name.into());
        self
    }

    /// 设置属性名称。
    pub fn with_property_name(mut self, property_name: impl Into<String>) -> Self {
        self.property_name = Some(property_name.into());
        self
    }

    /// 获取错误消息。
    pub fn message(&self) -> &str {
        &self.message
    }

    /// 获取方法名称。
    ///
    /// 对应 Java 的 `getMethod()`。
    pub fn method_name(&self) -> Option<&str> {
        self.method_name.as_deref()
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> Option<&str> {
        self.bean_name.as_deref()
    }

    /// 获取属性名称。
    pub fn property_name(&self) -> Option<&str> {
        self.property_name.as_deref()
    }

    /// 获取原始异常。
    ///
    /// 对应 Java 的 `getCause()`。
    pub fn cause(&self) -> Option<&(dyn std::error::Error + Send + Sync + 'static)> {
        self.cause.as_ref().map(|e| e.as_ref())
    }

    /// 构建详细的诊断消息。
    ///
    /// 包含 Bean 名称、方法名和属性名等上下文信息。
    pub fn detailed_message(&self) -> String {
        let mut parts = vec![self.message.clone()];
        if let Some(ref bean) = self.bean_name {
            parts.push(format!("bean='{}'", bean));
        }
        if let Some(ref method) = self.method_name {
            parts.push(format!("method='{}'", method));
        }
        if let Some(ref prop) = self.property_name {
            parts.push(format!("property='{}'", prop));
        }
        parts.join("; ")
    }
}

impl Clone for MethodInvocationException {
    fn clone(&self) -> Self {
        Self {
            message: self.message.clone(),
            method_name: self.method_name.clone(),
            bean_name: self.bean_name.clone(),
            property_name: self.property_name.clone(),
            // 注意：cause 无法 Clone，克隆时丢失异常链
            cause: None,
        }
    }
}

impl Default for MethodInvocationException {
    fn default() -> Self {
        Self::new("Method invocation failed")
    }
}

impl fmt::Display for MethodInvocationException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MethodInvocationException: {}", self.detailed_message())
    }
}

impl std::error::Error for MethodInvocationException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause.as_ref().map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_basic_exception() {
        let e = MethodInvocationException::new("test error");
        assert_eq!(e.message(), "test error");
        assert!(e.method_name().is_none());
        assert!(e.bean_name().is_none());
        assert!(e.property_name().is_none());
        assert!(e.cause().is_none());
    }

    #[test]
    fn with_cause_preserves_chain() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let e = MethodInvocationException::with_cause("wrapper", io_err);
        assert!(e.cause().is_some());
        assert_eq!(e.message(), "wrapper");
    }

    #[test]
    fn with_method_and_cause_full_context() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "bad");
        let e = MethodInvocationException::with_method_and_cause(
            "setter failed",
            "setName",
            io_err,
        );
        assert_eq!(e.method_name(), Some("setName"));
        assert!(e.cause().is_some());
    }

    #[test]
    fn builder_pattern_chaining() {
        let e = MethodInvocationException::new("error")
            .with_method("setAge")
            .with_bean_name("myBean")
            .with_property_name("age");

        assert_eq!(e.method_name(), Some("setAge"));
        assert_eq!(e.bean_name(), Some("myBean"));
        assert_eq!(e.property_name(), Some("age"));
    }

    #[test]
    fn detailed_message_includes_context() {
        let e = MethodInvocationException::new("failed")
            .with_method("setX")
            .with_bean_name("bean1")
            .with_property_name("x");
        let detail = e.detailed_message();
        assert!(detail.contains("failed"));
        assert!(detail.contains("bean='bean1'"));
        assert!(detail.contains("method='setX'"));
        assert!(detail.contains("property='x'"));
    }

    #[test]
    fn display_format() {
        let e = MethodInvocationException::new("oops");
        let s = format!("{}", e);
        assert!(s.starts_with("MethodInvocationException:"));
        assert!(s.contains("oops"));
    }

    #[test]
    fn error_trait_source_chain() {
        let cause = std::io::Error::new(std::io::ErrorKind::Other, "root");
        let e = MethodInvocationException::with_cause("wrap", cause);
        let err: &dyn std::error::Error = &e;
        assert!(err.source().is_some());
    }

    #[test]
    fn clone_preserves_fields_but_loses_cause() {
        let cause = std::io::Error::new(std::io::ErrorKind::Other, "x");
        let e = MethodInvocationException::with_cause("msg", cause)
            .with_method("m")
            .with_bean_name("b")
            .with_property_name("p");
        let cloned = e.clone();
        assert_eq!(cloned.message(), "msg");
        assert_eq!(cloned.method_name(), Some("m"));
        assert_eq!(cloned.bean_name(), Some("b"));
        assert_eq!(cloned.property_name(), Some("p"));
        // Clone 不保留 cause（因为 Box<dyn Error> 不实现 Clone）
        assert!(cloned.cause().is_none());
    }

    #[test]
    fn default_has_message() {
        let e = MethodInvocationException::default();
        assert!(!e.message().is_empty());
    }

    #[test]
    fn detailed_message_without_context() {
        let e = MethodInvocationException::new("basic");
        let detail = e.detailed_message();
        assert_eq!(detail, "basic");
    }
}
