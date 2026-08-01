//! 字符串 → 集合转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.StringToCollectionConverter`。

use crate::convert::{ConversionError, Convertible, Converter};

/// 字符串 → 集合转换器。
///
/// 对应 Java: org.springframework.core.convert.support.StringToCollectionConverter
///
/// Spring 语义：把逗号分隔的字符串切分后逐元素转换（对标
/// `StringUtils.commaDelimitedListToStringArray` + 元素级转换）。
pub struct StringToCollectionConverter;

impl<T: Convertible> Converter<&str, Vec<T>> for StringToCollectionConverter {
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
    fn converts_comma_delimited_list() {
        // A 类（合同对齐）：对标 Spring 逗号分隔 → 集合
        let converter = StringToCollectionConverter;
        let output: Vec<i32> = converter.convert("1, 2, 3").unwrap();
        assert_eq!(output, vec![1_i32, 2, 3]);
    }

    #[test]
    fn empty_string_yields_empty_collection() {
        // B 类（边界行为）：空字符串 → 空集合（对标 Spring 空列表）
        let converter = StringToCollectionConverter;
        let empty: Vec<i32> = converter.convert("").unwrap();
        assert!(empty.is_empty());
        let blank: Vec<i32> = converter.convert("  ").unwrap();
        assert!(blank.is_empty());
    }

    #[test]
    fn element_failure_propagates() {
        // C 类（错误路径）：任一元素转换失败则整体失败
        let converter = StringToCollectionConverter;
        let result: Result<Vec<i32>, _> = converter.convert("1, x, 3");
        let err = result.unwrap_err();
        assert_eq!(err.value, "x");
    }
}
