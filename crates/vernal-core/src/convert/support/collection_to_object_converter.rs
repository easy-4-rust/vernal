//! 集合 → 对象转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.CollectionToObjectConverter`。

use crate::convert::{ConversionError, Converter, Convertible};

/// 集合 → 对象转换器。
///
/// 对应 Java: org.springframework.core.convert.support.CollectionToObjectConverter
///
/// Spring 语义：把集合的**第一个元素**转换为目标类型；空集合报错。
pub struct CollectionToObjectConverter;

impl<S: ToString, T: Convertible> Converter<&[S], T> for CollectionToObjectConverter {
    fn convert(&self, source: &[S]) -> Result<T, ConversionError> {
        let Some(first) = source.first() else {
            return Err(ConversionError {
                value: String::new(),
                target_type: std::any::type_name::<T>(),
                reason: "空集合无法转换为单值".to_string(),
            });
        };
        T::from_str_value(&first.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_first_element() {
        // A 类（合同对齐）：对标 Spring 取首元素转换
        let converter = CollectionToObjectConverter;
        let value: i32 = converter.convert(&[7_i32, 8]).unwrap();
        assert_eq!(value, 7_i32);
    }

    #[test]
    fn empty_collection_returns_error() {
        // C 类（错误路径）
        let converter = CollectionToObjectConverter;
        let result: Result<i32, _> = converter.convert(&Vec::<String>::new());
        assert!(result.is_err());
    }
}
