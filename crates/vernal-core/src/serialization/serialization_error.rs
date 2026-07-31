//! 序列化错误。
//!
//! 对标 Spring `org.springframework.core.serializer.support.SerializationFailedException`。

/// 序列化错误。
///
/// 对应 Java: org.springframework.core.serializer.support.SerializationFailedException
#[derive(Debug)]
pub enum SerializationError {
    /// JSON 序列化失败（对标 jackson-databind 错误）
    Json(String),
    /// XML 序列化失败（对标 woodstox-core 错误）
    Xml(String),
    /// 其他序列化错误
    Other(String),
}

impl std::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(msg) => write!(f, "JSON 序列化失败: {msg}"),
            Self::Xml(msg) => write!(f, "XML 序列化失败: {msg}"),
            Self::Other(msg) => write!(f, "序列化失败: {msg}"),
        }
    }
}

impl std::error::Error for SerializationError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// 对标 Spring `SerializationFailedException` for JSON
    #[test]
    fn json_display_includes_failure_message() {
        let err = SerializationError::Json("unexpected EOF".to_string());
        let s = err.to_string();
        assert!(s.contains("JSON"), "should mention JSON: {s}");
        assert!(s.contains("unexpected EOF"), "should include message: {s}");
    }

    /// 对标 Spring `SerializationFailedException` for XML
    #[test]
    fn xml_display_includes_failure_message() {
        let err = SerializationError::Xml("invalid root".to_string());
        let s = err.to_string();
        assert!(s.contains("XML"), "should mention XML: {s}");
        assert!(s.contains("invalid root"), "should include message: {s}");
    }

    /// 通用序列化错误变体
    #[test]
    fn other_display_includes_failure_message() {
        let err = SerializationError::Other("codec failure".to_string());
        let s = err.to_string();
        assert!(s.contains("序列化失败"), "actual: {s}");
        assert!(s.contains("codec failure"), "actual: {s}");
    }

    /// 错误实现 std::error::Error 链路
    #[test]
    fn serialization_error_implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<SerializationError>();
    }

    /// Debug 应能派生（便于诊断日志输出）
    #[test]
    fn serialization_error_supports_debug() {
        let err = SerializationError::Json("x".to_string());
        let s = format!("{err:?}");
        assert!(s.contains("Json"));
    }
}
