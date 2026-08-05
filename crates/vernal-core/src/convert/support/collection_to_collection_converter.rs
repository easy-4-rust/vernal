//! 集合 → 集合转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.CollectionToCollectionConverter`。

use crate::convert::{ConversionError, Converter, Convertible};

/// 集合 → 集合转换器。
///
/// 对应 Java: org.springframework.core.convert.support.CollectionToCollectionConverter
///
/// Spring 语义：把源集合逐元素转换为目标集合元素类型。
pub struct CollectionToCollectionConverter;

impl<S: ToString, T: Convertible> Converter<&[S], Vec<T>> for CollectionToCollectionConverter {
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
    fn converts_collection_elements() {
        // A 类（合同对齐）
        let converter = CollectionToCollectionConverter;
        let output: Vec<i32> = converter.convert(&["1", "2"]).unwrap();
        assert_eq!(output, vec![1_i32, 2]);
    }

    #[test]
    fn preserves_empty_collection() {
        // B 类（边界行为）
        let converter = CollectionToCollectionConverter;
        let output: Vec<i32> = converter.convert(&Vec::<String>::new()).unwrap();
        assert!(output.is_empty());
    }
}
