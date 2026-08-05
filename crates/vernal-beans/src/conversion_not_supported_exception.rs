//! conversion_not_supported_exception — 对应 Java 异常类。
use std::fmt;

/// ConversionNotSupportedException 异常。
#[derive(Debug, Clone)]
pub struct ConversionNotSupportedException {
    message: String,
}

impl ConversionNotSupportedException {
    /// 创建一个新的实例。
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
    /// 获取消息。
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ConversionNotSupportedException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ConversionNotSupportedException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_string() {
        let err = ConversionNotSupportedException::new("conversion failed");
        assert_eq!(err.message(), "conversion failed");
    }

    #[test]
    fn test_new_with_string_owned() {
        let msg = String::from("owned conversion error");
        let err = ConversionNotSupportedException::new(msg);
        assert_eq!(err.message(), "owned conversion error");
    }

    #[test]
    fn test_display() {
        let err = ConversionNotSupportedException::new("display test");
        assert_eq!(format!("{}", err), "display test");
    }

    #[test]
    fn test_debug() {
        let err = ConversionNotSupportedException::new("debug test");
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("debug test"));
    }

    #[test]
    fn test_clone() {
        let err = ConversionNotSupportedException::new("clone test");
        let cloned = err.clone();
        assert_eq!(cloned.message(), "clone test");
    }

    #[test]
    fn test_error_trait() {
        let err = ConversionNotSupportedException::new("error trait test");
        let error: &dyn std::error::Error = &err;
        assert_eq!(error.to_string(), "error trait test");
    }

    #[test]
    fn test_new_with_empty_string() {
        let err = ConversionNotSupportedException::new("");
        assert_eq!(err.message(), "");
    }
}
