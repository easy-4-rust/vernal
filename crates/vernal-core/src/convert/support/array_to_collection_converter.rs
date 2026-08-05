//! 数组 → 集合转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.ArrayToCollectionConverter`。

use crate::convert::{ConversionError, Converter, Convertible};

/// 数组 → 集合转换器。
///
/// 对应 Java: org.springframework.core.convert.support.ArrayToCollectionConverter
///
/// Spring 语义：把数组逐元素转换为目标集合（Rust 中以 `Vec` 表达集合）。
pub struct ArrayToCollectionConverter;

impl<S: ToString, T: Convertible> Converter<&[S], Vec<T>> for ArrayToCollectionConverter {
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
    fn converts_array_to_collection() {
        // A 类（合同对齐）
        let converter = ArrayToCollectionConverter;
        let output: Vec<u8> = converter.convert(&["1", "2"]).unwrap();
        assert_eq!(output, vec![1_u8, 2]);
    }

    #[test]
    fn empty_array_yields_empty_collection() {
        // B 类（边界行为）
        let converter = ArrayToCollectionConverter;
        let output: Vec<u8> = converter.convert(&Vec::<String>::new()).unwrap();
        assert!(output.is_empty());
    }
}
