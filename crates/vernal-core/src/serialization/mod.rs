//! 序列化抽象模块（feature = "json" / "xml"）。
//!
//! 对标 Spring `jackson-databind`（JSON）和 `woodstox-core`（XML）。

/// 序列化错误类型。
#[derive(Debug, Clone)]
pub enum SerializationError {
    /// JSON 序列化/反序列化错误
    Json(String),
    /// XML 序列化/反序列化错误
    Xml(String),
    /// 通用序列化错误
    Other(String),
}

impl std::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(msg) => write!(f, "JSON error: {msg}"),
            Self::Xml(msg) => write!(f, "XML error: {msg}"),
            Self::Other(msg) => write!(f, "Serialization error: {msg}"),
        }
    }
}

impl std::error::Error for SerializationError {}

/// JSON 序列化/反序列化工具（feature = "json"）。
///
/// 对标 Spring `com.fasterxml.jackson.databind.ObjectMapper`。
///
/// 使用 `serde_json` crate 作为后端，提供 JSON 序列化/反序列化功能。
#[cfg(feature = "json")]
pub mod json {
    use super::SerializationError;

    /// JSON 序列化/反序列化工具。
    ///
    /// 对标 Spring `ObjectMapper`。
    #[derive(Debug, Clone)]
    pub struct JsonCodec;

    impl JsonCodec {
        /// 创建新的 JSON 编解码器。
        pub fn new() -> Self {
            Self
        }

        /// 序列化对象为 JSON 字符串。
        pub fn to_string<T: serde::Serialize>(&self, value: &T) -> Result<String, SerializationError> {
            serde_json::to_string(value).map_err(|e| SerializationError::Json(e.to_string()))
        }

        /// 序列化对象为格式化的 JSON 字符串。
        pub fn to_string_pretty<T: serde::Serialize>(&self, value: &T) -> Result<String, SerializationError> {
            serde_json::to_string_pretty(value).map_err(|e| SerializationError::Json(e.to_string()))
        }

        /// 反序列化 JSON 字符串为对象。
        pub fn from_str<T: serde::de::DeserializeOwned>(&self, json: &str) -> Result<T, SerializationError> {
            serde_json::from_str(json).map_err(|e| SerializationError::Json(e.to_string()))
        }

        /// 序列化对象为 JSON 字节向量。
        pub fn to_vec<T: serde::Serialize>(&self, value: &T) -> Result<Vec<u8>, SerializationError> {
            serde_json::to_vec(value).map_err(|e| SerializationError::Json(e.to_string()))
        }

        /// 反序列化 JSON 字节向量为对象。
        pub fn from_slice<T: serde::de::DeserializeOwned>(&self, slice: &[u8]) -> Result<T, SerializationError> {
            serde_json::from_slice(slice).map_err(|e| SerializationError::Json(e.to_string()))
        }

        /// 反序列化 JSON 值为 serde_json::Value。
        pub fn to_value<T: serde::Serialize>(&self, value: &T) -> Result<serde_json::Value, SerializationError> {
            serde_json::to_value(value).map_err(|e| SerializationError::Json(e.to_string()))
        }

        /// 反序列化 serde_json::Value 为对象。
        pub fn from_value<T: serde::de::DeserializeOwned>(&self, value: serde_json::Value) -> Result<T, SerializationError> {
            serde_json::from_value(value).map_err(|e| SerializationError::Json(e.to_string()))
        }
    }

    impl Default for JsonCodec {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialization_error_display() {
        let err = SerializationError::Json("invalid json".to_string());
        assert!(err.to_string().contains("JSON error"));
    }

    #[test]
    fn serialization_error_is_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<SerializationError>();
    }
}

/// XML 序列化/反序列化工具（feature = "xml"）。
///
/// 对标 Spring `com.fasterxml.woodstox.core`。
///
/// 使用 `quick-xml` crate 作为后端，提供 XML 序列化/反序列化功能。
#[cfg(feature = "xml")]
pub mod xml {
    use super::SerializationError;

    /// XML 序列化/反序列化工具。
    ///
    /// 对标 Spring `WoodstoxObjectMapper`。
    #[derive(Debug, Clone)]
    pub struct XmlCodec;

    impl XmlCodec {
        /// 创建新的 XML 编解码器。
        pub fn new() -> Self {
            Self
        }

        /// 序列化对象为 XML 字符串。
        pub fn to_string<T: serde::Serialize>(&self, value: &T) -> Result<String, SerializationError> {
            quick_xml::se::to_string(value).map_err(|e| SerializationError::Xml(e.to_string()))
        }

        /// 反序列化 XML 字符串为对象。
        pub fn from_str<T: serde::de::DeserializeOwned>(&self, xml: &str) -> Result<T, SerializationError> {
            quick_xml::de::from_str(xml).map_err(|e| SerializationError::Xml(e.to_string()))
        }
    }

    impl Default for XmlCodec {
        fn default() -> Self {
            Self::new()
        }
    }
}
