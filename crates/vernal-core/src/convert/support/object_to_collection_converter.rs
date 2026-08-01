//! 对象 → 集合转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.ObjectToCollectionConverter`。

use crate::convert::{ConversionError, Convertible, Converter};

/// 对象 → 集合转换器。
///
/// 对应 Java: org.springframework.core.convert.support.ObjectToCollectionConverter
///
/// Spring 语义：把单值对象包装为单元素集合。
pub struct ObjectToCollectionConverter;

impl<T: Convertible> Converter<&str, Vec<T>> for ObjectToCollectionConverter {
    fn convert(&self, source: &str) -> Result<Vec<T>, ConversionError> {
        T::from_str_value(source).map(|value| vec![value])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_single_value_into_collection() {
        // A 类（合同对齐）
        let converter = ObjectToCollectionConverter;
        let output: Vec<bool> = converter.convert("true").unwrap();
        assert_eq!(output, vec![true]);
    }

    #[test]
    fn invalid_value_returns_error() {
        // C 类（错误路径）
        let converter = ObjectToCollectionConverter;
        let result: Result<Vec<bool>, _> = converter.convert("maybe");
        assert!(result.is_err());
    }
}
