//! bean_instantiation_exception — 对应 Java 异常类。

use std::fmt;

/// BeanInstantiationException 异常。
#[derive(Debug, Clone)]
pub struct BeanInstantiationException {
    message: String,
}

impl BeanInstantiationException {
    /// 创建一个新的实例。
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
    /// 获取消息。
    pub fn message(&self) -> &str { &self.message }
}

impl fmt::Display for BeanInstantiationException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BeanInstantiationException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_string() {
        let err = BeanInstantiationException::new("test error");
        assert_eq!(err.message(), "test error");
    }

    #[test]
    fn test_new_with_string_owned() {
        let msg = String::from("owned error message");
        let err = BeanInstantiationException::new(msg);
        assert_eq!(err.message(), "owned error message");
    }

    #[test]
    fn test_display() {
        let err = BeanInstantiationException::new("display test");
        assert_eq!(format!("{}", err), "display test");
    }

    #[test]
    fn test_debug() {
        let err = BeanInstantiationException::new("debug test");
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("debug test"));
    }

    #[test]
    fn test_clone() {
        let err = BeanInstantiationException::new("clone test");
        let cloned = err.clone();
        assert_eq!(cloned.message(), "clone test");
    }

    #[test]
    fn test_error_trait() {
        let err = BeanInstantiationException::new("error trait test");
        let error: &dyn std::error::Error = &err;
        assert_eq!(error.to_string(), "error trait test");
    }

    #[test]
    fn test_new_with_empty_string() {
        let err = BeanInstantiationException::new("");
        assert_eq!(err.message(), "");
    }
}
