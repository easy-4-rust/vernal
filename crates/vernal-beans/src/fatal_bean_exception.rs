//! fatal_bean_exception — 对应 Java 异常类。

use std::fmt;

/// FatalBeanException 异常。
#[derive(Debug, Clone)]
pub struct FatalBeanException {
    message: String,
}

impl FatalBeanException {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
    pub fn message(&self) -> &str { &self.message }
}

impl fmt::Display for FatalBeanException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for FatalBeanException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_string() {
        let err = FatalBeanException::new("fatal error");
        assert_eq!(err.message(), "fatal error");
    }

    #[test]
    fn test_new_with_string_owned() {
        let msg = String::from("owned fatal error");
        let err = FatalBeanException::new(msg);
        assert_eq!(err.message(), "owned fatal error");
    }

    #[test]
    fn test_display() {
        let err = FatalBeanException::new("display test");
        assert_eq!(format!("{}", err), "display test");
    }

    #[test]
    fn test_debug() {
        let err = FatalBeanException::new("debug test");
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("debug test"));
    }

    #[test]
    fn test_clone() {
        let err = FatalBeanException::new("clone test");
        let cloned = err.clone();
        assert_eq!(cloned.message(), "clone test");
    }

    #[test]
    fn test_error_trait() {
        let err = FatalBeanException::new("error trait test");
        let error: &dyn std::error::Error = &err;
        assert_eq!(error.to_string(), "error trait test");
    }

    #[test]
    fn test_new_with_empty_string() {
        let err = FatalBeanException::new("");
        assert_eq!(err.message(), "");
    }
}
