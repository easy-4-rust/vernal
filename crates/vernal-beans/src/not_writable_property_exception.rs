//! not_writable_property_exception — 对应 Java 异常类。

use std::fmt;

/// NotWritablePropertyException 异常。
#[derive(Debug, Clone)]
pub struct NotWritablePropertyException {
    message: String,
}

impl NotWritablePropertyException {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
    pub fn message(&self) -> &str { &self.message }
}

impl fmt::Display for NotWritablePropertyException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for NotWritablePropertyException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_string() {
        let err = NotWritablePropertyException::new("not writable");
        assert_eq!(err.message(), "not writable");
    }

    #[test]
    fn test_new_with_string_owned() {
        let msg = String::from("owned not writable");
        let err = NotWritablePropertyException::new(msg);
        assert_eq!(err.message(), "owned not writable");
    }

    #[test]
    fn test_display() {
        let err = NotWritablePropertyException::new("display test");
        assert_eq!(format!("{}", err), "display test");
    }

    #[test]
    fn test_debug() {
        let err = NotWritablePropertyException::new("debug test");
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("debug test"));
    }

    #[test]
    fn test_clone() {
        let err = NotWritablePropertyException::new("clone test");
        let cloned = err.clone();
        assert_eq!(cloned.message(), "clone test");
    }

    #[test]
    fn test_error_trait() {
        let err = NotWritablePropertyException::new("error trait test");
        let error: &dyn std::error::Error = &err;
        assert_eq!(error.to_string(), "error trait test");
    }

    #[test]
    fn test_new_with_empty_string() {
        let err = NotWritablePropertyException::new("");
        assert_eq!(err.message(), "");
    }
}
