//! type_mismatch_exception — 对应 Java 异常类。
use std::fmt;

/// TypeMismatchException 异常。
///
/// 对应 Java 类：`org.springframework.beans.TypeMismatchException`。
///
/// 当类型转换失败时抛出，包含错误消息、目标类型和实际值信息。
#[derive(Debug, Clone)]
pub struct TypeMismatchException {
    message: String,
    target_type: Option<String>,
    actual_value: Option<String>,
}

impl TypeMismatchException {
    /// 创建新的 TypeMismatchException（仅包含消息）。
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            target_type: None,
            actual_value: None,
        }
    }

    /// 创建带有目标类型和实际值的 TypeMismatchException。
    pub fn with_details(message: impl Into<String>, target_type: String, actual_value: String) -> Self {
        Self {
            message: message.into(),
            target_type: Some(target_type),
            actual_value: Some(actual_value),
        }
    }

    /// 获取错误消息。
    pub fn message(&self) -> &str { &self.message }

    /// 获取目标类型（如果有）。
    pub fn target_type(&self) -> Option<&str> { self.target_type.as_deref() }

    /// 获取实际值（如果有）。
    pub fn actual_value(&self) -> Option<&str> { self.actual_value.as_deref() }
}

impl fmt::Display for TypeMismatchException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TypeMismatchException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_string() {
        let err = TypeMismatchException::new("type mismatch");
        assert_eq!(err.message(), "type mismatch");
        assert_eq!(err.target_type(), None);
        assert_eq!(err.actual_value(), None);
    }

    #[test]
    fn test_new_with_string_owned() {
        let msg = String::from("owned type mismatch");
        let err = TypeMismatchException::new(msg);
        assert_eq!(err.message(), "owned type mismatch");
    }

    #[test]
    fn test_with_details() {
        let err = TypeMismatchException::with_details(
            "conversion failed".to_string(),
            "i32".to_string(),
            "abc".to_string(),
        );
        assert_eq!(err.message(), "conversion failed");
        assert_eq!(err.target_type(), Some("i32"));
        assert_eq!(err.actual_value(), Some("abc"));
    }

    #[test]
    fn test_display() {
        let err = TypeMismatchException::new("display test");
        assert_eq!(format!("{}", err), "display test");
    }

    #[test]
    fn test_debug() {
        let err = TypeMismatchException::with_details(
            "debug test".to_string(),
            "String".to_string(),
            "42".to_string(),
        );
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("debug test"));
        assert!(debug_str.contains("String"));
    }

    #[test]
    fn test_clone() {
        let err = TypeMismatchException::with_details(
            "clone test".to_string(),
            "f64".to_string(),
            "3.14".to_string(),
        );
        let cloned = err.clone();
        assert_eq!(cloned.message(), "clone test");
        assert_eq!(cloned.target_type(), Some("f64"));
        assert_eq!(cloned.actual_value(), Some("3.14"));
    }

    #[test]
    fn test_error_trait() {
        let err = TypeMismatchException::new("error trait test");
        let error: &dyn std::error::Error = &err;
        assert_eq!(error.to_string(), "error trait test");
    }

    #[test]
    fn test_new_with_empty_string() {
        let err = TypeMismatchException::new("");
        assert_eq!(err.message(), "");
    }

    #[test]
    fn test_with_details_empty_strings() {
        let err = TypeMismatchException::with_details(
            "".to_string(),
            "".to_string(),
            "".to_string(),
        );
        assert_eq!(err.message(), "");
        assert_eq!(err.target_type(), Some(""));
        assert_eq!(err.actual_value(), Some(""));
    }
}
