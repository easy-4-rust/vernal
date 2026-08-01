//! 数组 → 字符串转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.ArrayToStringConverter`。

use crate::convert::{ConversionError, Converter};

/// 数组 → 字符串转换器。
///
/// 对应 Java: org.springframework.core.convert.support.ArrayToStringConverter
///
/// Spring 语义：把数组元素逐个转换后以逗号拼接（对标
/// `StringUtils.arrayToCommaDelimitedString`）。
pub struct ArrayToStringConverter;

impl<T: std::fmt::Display> Converter<&[T], String> for ArrayToStringConverter {
    fn convert(&self, source: &[T]) -> Result<String, ConversionError> {
        let parts: Vec<String> = source.iter().map(ToString::to_string).collect();
        Ok(parts.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_array_elements() {
        // A 类（合同对齐）：对标 Spring 数组 → 逗号分隔字符串
        let converter = ArrayToStringConverter;
        assert_eq!(converter.convert(&["a", "b"]).unwrap(), "a,b");
    }

    #[test]
    fn empty_array_yields_empty_string() {
        // B 类（边界行为）
        let converter = ArrayToStringConverter;
        assert_eq!(converter.convert(&Vec::<u8>::new()).unwrap(), "");
    }
}
