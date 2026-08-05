//! JSON 编解码器。
//!
//! 对标 Spring `org.springframework.http.converter.json.MappingJackson2HttpMessageConverter`。
//! 底层使用 `serde_json`（对标 jackson-databind）。

#![cfg(feature = "json")]

use std::any::TypeId;
use std::collections::HashMap;

use serde::Serialize;
use serde::de::DeserializeOwned;

use super::serialization_error::SerializationError;

/// JSON 编解码器。
///
/// 对应 Java: `MappingJackson2HttpMessageConverter`（简化版）
pub struct JsonCodec;

impl JsonCodec {
    /// 创建新的 JSON 编解码器。
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// 将字符串序列化为 JSON。
    ///
    /// # Errors
    ///
    /// 当 type T 不满足 Serialize trait 时返回 `SerializationError::TypeMismatch`。
    pub fn to_string<T: Serialize>(&self, value: &T) -> Result<String, SerializationError> {
        serde_json::to_string(value).map_err(|e| SerializationError::Json(e.to_string()))
    }

    /// 美化输出。
    pub fn to_string_pretty<T: Serialize>(&self, value: &T) -> Result<String, SerializationError> {
        serde_json::to_string_pretty(value).map_err(|e| SerializationError::Json(e.to_string()))
    }

    /// 反序列化 JSON 字符串。
    pub fn from_str<T: DeserializeOwned>(&self, json: &str) -> Result<T, SerializationError> {
        serde_json::from_str(json).map_err(|e| SerializationError::Json(e.to_string()))
    }

    /// 序列化为字节。
    pub fn to_vec<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, SerializationError> {
        serde_json::to_vec(value).map_err(|e| SerializationError::Json(e.to_string()))
    }

    /// 从字节反序列化。
    pub fn from_slice<T: DeserializeOwned>(&self, slice: &[u8]) -> Result<T, SerializationError> {
        serde_json::from_slice(slice).map_err(|e| SerializationError::Json(e.to_string()))
    }

    /// 序列化为 serde_json Value。
    pub fn to_value<T: Serialize>(
        &self,
        value: &T,
    ) -> Result<serde_json::Value, SerializationError> {
        serde_json::to_value(value).map_err(|e| SerializationError::Json(e.to_string()))
    }

    /// 从 serde_json Value 反序列化。
    pub fn from_value<T: DeserializeOwned>(
        &self,
        value: serde_json::Value,
    ) -> Result<T, SerializationError> {
        serde_json::from_value(value).map_err(|e| SerializationError::Json(e.to_string()))
    }
}

impl Default for JsonCodec {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    #[test]
    fn roundtrip_string() {
        let codec = JsonCodec::new();
        let original = TestStruct {
            name: "test".to_string(),
            value: 42,
        };
        let json = codec.to_string(&original).unwrap();
        let parsed: TestStruct = codec.from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn roundtrip_pretty() {
        let codec = JsonCodec::new();
        let original = TestStruct {
            name: "demo".to_string(),
            value: 100,
        };
        let json = codec.to_string_pretty(&original).unwrap();
        assert!(json.contains("\n")); // pretty 输出含换行
        let parsed: TestStruct = codec.from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn roundtrip_vec() {
        let codec = JsonCodec::new();
        let original = vec![1, 2, 3, 4, 5];
        let bytes = codec.to_vec(&original).unwrap();
        let parsed: Vec<i32> = codec.from_slice(&bytes).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn rounttrip_primitive() {
        let codec = JsonCodec::new();
        let json = "42".to_string();
        let parsed: i32 = codec.from_str(&json).unwrap();
        assert_eq!(parsed, 42);
    }

    #[test]
    fn roundtrip_map() {
        let codec = JsonCodec::new();
        let mut original = HashMap::new();
        original.insert("key1".to_string(), "value1".to_string());
        let json = codec.to_string(&original).unwrap();
        let parsed: HashMap<String, String> = codec.from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn roundtrip_value() {
        let codec = JsonCodec::new();
        let original = TestStruct {
            name: "value".to_string(),
            value: 7,
        };
        let value = codec.to_value(&original).unwrap();
        let parsed: TestStruct = codec.from_value(value).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn invalid_json_returns_error() {
        let codec = JsonCodec::new();
        let result: Result<TestStruct, _> = codec.from_str("{invalid json");
        assert!(result.is_err());
    }

    #[test]
    fn type_mismatch_returns_error() {
        let codec = JsonCodec::new();
        // 字符串无法解析为 struct
        let result: Result<TestStruct, _> = codec.from_str("\"just a string\"");
        assert!(result.is_err());
    }

    #[test]
    fn empty_object_roundtrip() {
        let codec = JsonCodec::new();
        let original = TestStruct {
            name: "".to_string(),
            value: 0,
        };
        let json = codec.to_string(&original).unwrap();
        let parsed: TestStruct = codec.from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn default_impl_works() {
        let _codec: JsonCodec = JsonCodec::default();
    }
}
