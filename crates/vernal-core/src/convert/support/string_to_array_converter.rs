//! 字符串 → 数组转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.StringToArrayConverter`。

use crate::convert::{ConversionError, Converter, Convertible};

/// 字符串 → 数组转换器。
///
/// 对应 Java: org.springframework.core.convert.support.StringToArrayConverter
///
/// Spring 语义：把逗号分隔的字符串转换为数组（Rust 中以 `Vec` 表达数组）。
pub struct StringToArrayConverter;

impl<T: Convertible> Converter<&str, Vec<T>> for StringToArrayConverter {
    fn convert(&self, source: &str) -> Result<Vec<T>, ConversionError> {
        if source.trim().is_empty() {
            return Ok(Vec::new());
        }
        source
            .split(',')
            .map(|part| T::from_str_value(part.trim()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_comma_delimited_array() {
        // A 类（合同对齐）：对标 Spring 字符串 → 数组
        let converter = StringToArrayConverter;
        let output: Vec<String> = converter.convert("a, b").unwrap();
        assert_eq!(output, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn empty_string_yields_empty_array() {
        // B 类（边界行为）
        let converter = StringToArrayConverter;
        let output: Vec<u8> = converter.convert("").unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn element_failure_propagates() {
        // C 类（错误路径）
        let converter = StringToArrayConverter;
        let result: Result<Vec<u8>, _> = converter.convert("1, 999");
        assert!(result.is_err());
    }
}
