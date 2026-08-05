//! 对象 → 数组转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.ObjectToArrayConverter`。

use crate::convert::{ConversionError, Converter, Convertible};

/// 对象 → 数组转换器。
///
/// 对应 Java: org.springframework.core.convert.support.ObjectToArrayConverter
///
/// Spring 语义：把单值对象包装为单元素数组。
pub struct ObjectToArrayConverter;

impl<T: Convertible> Converter<&str, Vec<T>> for ObjectToArrayConverter {
    fn convert(&self, source: &str) -> Result<Vec<T>, ConversionError> {
        T::from_str_value(source).map(|value| vec![value])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_single_value_into_array() {
        // A 类（合同对齐）：对标 Spring 单值 → 单元素数组
        let converter = ObjectToArrayConverter;
        let output: Vec<i32> = converter.convert("42").unwrap();
        assert_eq!(output, vec![42_i32]);
    }

    #[test]
    fn invalid_value_returns_error() {
        // C 类（错误路径）
        let converter = ObjectToArrayConverter;
        let result: Result<Vec<u32>, _> = converter.convert("x");
        assert!(result.is_err());
    }
}
