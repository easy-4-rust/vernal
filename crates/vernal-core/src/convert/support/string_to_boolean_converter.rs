//! 字符串 → 布尔转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.StringToBooleanConverter`。

use crate::convert::{ConversionError, Converter, Convertible};

/// 字符串 → 布尔转换器。
///
/// 对应 Java: org.springframework.core.convert.support.StringToBooleanConverter
///
/// 接受的字符串值（大小写不敏感）：`true`/`1`/`yes`/`on` → `true`；
/// `false`/`0`/`no`/`off` → `false`。
pub struct StringToBooleanConverter;

impl Converter<&str, bool> for StringToBooleanConverter {
    fn convert(&self, source: &str) -> Result<bool, ConversionError> {
        bool::from_str_value(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_truthy_values() {
        // A 类（合同对齐）：对标 Spring 的 `"true"/"1"/"yes"/"on"` 判定
        let converter = StringToBooleanConverter;
        for value in ["true", "1", "yes", "on", "TRUE", "Yes"] {
            assert!(converter.convert(value).unwrap(), "value: {value}");
        }
    }

    #[test]
    fn converts_falsy_values() {
        // B 类（边界行为）：空字符串与未知值应报错而非默认 false
        let converter = StringToBooleanConverter;
        for value in ["false", "0", "no", "off"] {
            assert!(!converter.convert(value).unwrap(), "value: {value}");
        }
    }

    #[test]
    fn invalid_value_returns_error() {
        // C 类（错误路径）：对标 Spring `ConversionFailedException` 语义
        let converter = StringToBooleanConverter;
        let err = converter.convert("invalid").unwrap_err();
        assert_eq!(err.target_type, "bool");
        assert!(err.reason.contains("期望"));
    }
}
