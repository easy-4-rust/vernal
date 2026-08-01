//! 集合 → 数组转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.CollectionToArrayConverter`。

use crate::convert::{ConversionError, Convertible, Converter};

/// 集合 → 数组转换器。
///
/// 对应 Java: org.springframework.core.convert.support.CollectionToArrayConverter
///
/// Spring 语义：把集合逐元素转换为目标数组类型（Rust 中以 `Vec` 表达数组）。
pub struct CollectionToArrayConverter;

impl<S: ToString, T: Convertible> Converter<&[S], Vec<T>> for CollectionToArrayConverter {
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
    fn converts_collection_to_array() {
        // A 类（合同对齐）
        let converter = CollectionToArrayConverter;
        let output: Vec<u8> = converter.convert(&[10_i32, 20]).unwrap();
        assert_eq!(output, vec![10_u8, 20]);
    }

    #[test]
    fn element_failure_propagates() {
        // C 类（错误路径）
        let converter = CollectionToArrayConverter;
        let result: Result<Vec<u8>, _> = converter.convert(&[300_i32]);
        assert!(result.is_err());
    }
}
