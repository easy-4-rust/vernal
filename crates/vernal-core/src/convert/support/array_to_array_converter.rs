//! 数组 → 数组转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.ArrayToArrayConverter`。

use crate::convert::{ConversionError, Convertible, Converter};

/// 数组 → 数组转换器。
///
/// 对应 Java: org.springframework.core.convert.support.ArrayToArrayConverter
///
/// Spring 语义：逐元素转换（元素级转换失败则整体失败）。
pub struct ArrayToArrayConverter;

impl<S: ToString, T: Convertible> Converter<&[S], Vec<T>> for ArrayToArrayConverter {
    fn convert(&self, source: &[S]) -> Result<Vec<T>, ConversionError> {
        source
            .iter()
            .map(|item| T::from_str_value(&item.to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_elements() {
        // A 类（合同对齐）：对标 Spring 数组元素级转换
        let converter = ArrayToArrayConverter;
        let output: Vec<String> = converter.convert(&[1_i32, 2, 3]).unwrap();
        assert_eq!(output, vec!["1".to_string(), "2".to_string(), "3".to_string()]);
    }

    #[test]
    fn empty_array_converts_to_empty() {
        // B 类（边界行为）
        let converter = ArrayToArrayConverter;
        let output: Vec<String> = converter.convert(&Vec::<i32>::new()).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn element_failure_propagates() {
        // C 类（错误路径）
        let converter = ArrayToArrayConverter;
        let result: Result<Vec<u8>, _> = converter.convert(&[256_i32]);
        assert!(result.is_err());
    }
}
