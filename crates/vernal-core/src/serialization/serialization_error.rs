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
