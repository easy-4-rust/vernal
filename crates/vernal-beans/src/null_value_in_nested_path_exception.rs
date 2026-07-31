//! null_value_in_nested_path_exception — 对应 Java 异常类。

use std::fmt;

/// NullValueInNestedPathException 异常。
#[derive(Debug, Clone)]
pub struct NullValueInNestedPathException {
    message: String,
}

impl NullValueInNestedPathException {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
    pub fn message(&self) -> &str { &self.message }
}

impl fmt::Display for NullValueInNestedPathException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for NullValueInNestedPathException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_string() {
        let err = NullValueInNestedPathException::new("null in nested path");
        assert_eq!(err.message(), "null in nested path");
    }

    #[test]
    fn test_new_with_string_owned() {
        let msg = String::from("owned null path error");
        let err = NullValueInNestedPathException::new(msg);
        assert_eq!(err.message(), "owned null path error");
    }

    #[test]
    fn test_display() {
        let err = NullValueInNestedPathException::new("display test");
        assert_eq!(format!("{}", err), "display test");
    }

    #[test]
    fn test_debug() {
        let err = NullValueInNestedPathException::new("debug test");
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("debug test"));
    }

    #[test]
    fn test_clone() {
        let err = NullValueInNestedPathException::new("clone test");
        let cloned = err.clone();
        assert_eq!(cloned.message(), "clone test");
    }

    #[test]
    fn test_error_trait() {
        let err = NullValueInNestedPathException::new("error trait test");
        let error: &dyn std::error::Error = &err;
        assert_eq!(error.to_string(), "error trait test");
    }

    #[test]
    fn test_new_with_empty_string() {
        let err = NullValueInNestedPathException::new("");
        assert_eq!(err.message(), "");
    }
}
