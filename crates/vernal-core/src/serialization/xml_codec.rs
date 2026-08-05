//! XML 编解码器。
//!
//! 对标 Spring `org.springframework.http.converter.xml.MappingJackson2XmlHttpMessageConverter`。
//! 底层使用 `quick-xml`（对标 woodstox-core）。

#![cfg(feature = "xml")]

use serde::Serialize;
use serde::de::DeserializeOwned;

use super::serialization_error::SerializationError;

/// XML 编解码器。
///
/// 对应 Java: `MappingJackson2XmlHttpMessageConverter`（简化版）
pub struct XmlCodec;

impl XmlCodec {
    /// 创建新的 XML 编解码器。
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// 序列化为 XML 字符串。
    ///
    /// # Errors
    ///
    /// 序列化失败时返回 `SerializationError`
    pub fn to_string<T: Serialize>(&self, value: &T) -> Result<String, SerializationError> {
        quick_xml::se::to_string(value).map_err(|e| SerializationError::Xml(e.to_string()))
    }

    /// 从 XML 字符串反序列化。
    pub fn from_str<T: DeserializeOwned>(&self, xml: &str) -> Result<T, SerializationError> {
        quick_xml::de::from_str(xml).map_err(|e| SerializationError::Xml(e.to_string()))
    }
}

impl Default for XmlCodec {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct XmlStruct {
        name: String,
    }

    #[test]
    fn roundtrip_simple() {
        let codec = XmlCodec::new();
        let original = XmlStruct {
            name: "test".to_string(),
        };
        let xml = codec.to_string(&original).unwrap();
        let parsed: XmlStruct = codec.from_str(&xml).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn invalid_xml_returns_error() {
        let codec = XmlCodec::new();
        let result: Result<XmlStruct, _> = codec.from_str("<invalid");
        assert!(result.is_err());
    }

    #[test]
    fn default_impl_works() {
        let _codec: XmlCodec = XmlCodec::default();
    }
}
