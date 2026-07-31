//! property_batch_update_exception — 对应 Java 异常类。
use std::fmt;

/// PropertyBatchUpdateException 异常。
#[derive(Debug, Clone)]
pub struct PropertyBatchUpdateException {
    message: String,
}

impl PropertyBatchUpdateException {
    /// 创建一个新的实例。
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
    /// 获取消息。
    pub fn message(&self) -> &str { &self.message }
}

impl fmt::Display for PropertyBatchUpdateException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PropertyBatchUpdateException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_string() {
        let err = PropertyBatchUpdateException::new("batch update error");
        assert_eq!(err.message(), "batch update error");
    }

    #[test]
    fn test_new_with_string_owned() {
        let msg = String::from("owned batch update error");
        let err = PropertyBatchUpdateException::new(msg);
        assert_eq!(err.message(), "owned batch update error");
    }

    #[test]
    fn test_display() {
        let err = PropertyBatchUpdateException::new("display test");
        assert_eq!(format!("{}", err), "display test");
    }

    #[test]
    fn test_debug() {
        let err = PropertyBatchUpdateException::new("debug test");
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("debug test"));
    }

    #[test]
    fn test_clone() {
        let err = PropertyBatchUpdateException::new("clone test");
        let cloned = err.clone();
        assert_eq!(cloned.message(), "clone test");
    }

    #[test]
    fn test_error_trait() {
        let err = PropertyBatchUpdateException::new("error trait test");
        let error: &dyn std::error::Error = &err;
        assert_eq!(error.to_string(), "error trait test");
    }

    #[test]
    fn test_new_with_empty_string() {
        let err = PropertyBatchUpdateException::new("");
        assert_eq!(err.message(), "");
    }
}
