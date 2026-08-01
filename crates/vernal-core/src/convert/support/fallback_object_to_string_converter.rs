//! 兜底对象 → 字符串转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.FallbackObjectToStringConverter`。

use crate::convert::{ConversionError, Converter};

/// 兜底对象 → 字符串转换器。
///
/// 对应 Java: org.springframework.core.convert.support.FallbackObjectToStringConverter
///
/// Spring 语义：作为转换链的兜底项，任意对象 → `String`（`toString()`）；
/// Rust 中以 `Display` 表达，对 `Display` 不可用的类型保持错误。
pub struct FallbackObjectToStringConverter;

impl<T: std::fmt::Display> Converter<&T, String> for FallbackObjectToStringConverter {
    fn convert(&self, source: &T) -> Result<String, ConversionError> {
        Ok(source.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_converts_any_display_type() {
        // A 类（合同对齐）：对标 Spring 兜底转换
        let converter = FallbackObjectToStringConverter;
        assert_eq!(converter.convert(&true).unwrap(), "true");
        assert_eq!(converter.convert(&3.25_f64).unwrap(), "3.25");
    }

    #[test]
    fn works_alongside_object_to_string_converter() {
        // D 类（重构安全）：两个转换器行为一致
        let fallback = FallbackObjectToStringConverter;
        let object = crate::convert::support::ObjectToStringConverter;
        assert_eq!(
            fallback.convert(&"x").unwrap(),
            object.convert(&"x").unwrap()
        );
    }
}
