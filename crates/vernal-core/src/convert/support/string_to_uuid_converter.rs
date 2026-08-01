//! 字符串 → UUID 转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.StringToUUIDConverter`。
//! 仅在 feature `convert-uuid` 下编译。

use crate::convert::{ConversionError, Convertible, Converter};

/// 字符串 → UUID 转换器。
///
/// 对应 Java: org.springframework.core.convert.support.StringToUUIDConverter
pub struct StringToUUIDConverter;

impl Converter<&str, uuid::Uuid> for StringToUUIDConverter {
    fn convert(&self, source: &str) -> Result<uuid::Uuid, ConversionError> {
        uuid::Uuid::from_str_value(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_valid_uuid() {
        // A 类（合同对齐）：对标 Spring `UUID.fromString`
        let converter = StringToUUIDConverter;
        let parsed = converter
            .convert("550e8400-e29b-41d4-a716-446655440000")
            .unwrap();
        assert_eq!(parsed.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn invalid_uuid_returns_error() {
        // C 类（错误路径）
        let converter = StringToUUIDConverter;
        assert!(converter.convert("not-a-uuid").is_err());
    }
}
