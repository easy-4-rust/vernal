//! UUID 类型转换器(feature = "convert-uuid")。
//!
//! 对标 Spring 的 `StringToUUIDConverter`,将字符串解析为 [`uuid::Uuid`]。
//!
//! # 启用方式
//!
//! ```toml
//! [dependencies]
//! vernal-core = { features = ["convert-uuid"] }
//! ```

use super::{ConversionError, Convertible};

/// UUID 转换器。
///
/// 对标 Spring `StringToUUIDConverter`。支持标准 UUID 格式
/// （如 `550e8400-e29b-41d4-a716-446655440000`）。
impl Convertible for uuid::Uuid {
    fn from_str_value(value: &str) -> Result<Self, ConversionError> {
        value.parse::<uuid::Uuid>().map_err(|e| ConversionError {
            value: value.to_string(),
            target_type: "Uuid",
            reason: format!("UUID 解析失败: {e}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_uuid() {
        let uuid = uuid::Uuid::from_str_value("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(uuid.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn parse_invalid_uuid() {
        let result = uuid::Uuid::from_str_value("not-a-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn round_trip() {
        let original = uuid::Uuid::new_v4();
        let s = original.to_string();
        let parsed = uuid::Uuid::from_str_value(&s).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn nil_uuid() {
        let uuid = uuid::Uuid::from_str_value("00000000-0000-0000-0000-000000000000").unwrap();
        assert_eq!(uuid, uuid::Uuid::nil());
    }
}
