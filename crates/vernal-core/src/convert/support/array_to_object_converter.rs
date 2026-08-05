//! 数组 → 对象转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.ArrayToObjectConverter`。

use crate::convert::{ConversionError, Converter, Convertible};

/// 数组 → 对象转换器。
///
/// 对应 Java: org.springframework.core.convert.support.ArrayToObjectConverter
///
/// Spring 语义：把单元素数组转换为元素类型；空数组报错
/// （"Cannot convert empty array to..."）。
pub struct ArrayToObjectConverter;

impl<S: ToString, T: Convertible> Converter<&[S], T> for ArrayToObjectConverter {
    fn convert(&self, source: &[S]) -> Result<T, ConversionError> {
        let Some(first) = source.first() else {
            return Err(ConversionError {
                value: String::new(),
                target_type: std::any::type_name::<T>(),
                reason: "空数组无法转换为单值".to_string(),
            });
        };
        T::from_str_value(&first.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_single_element_array() {
        // A 类（合同对齐）：对标 Spring 单元素数组 → 元素
        let converter = ArrayToObjectConverter;
        let value: i64 = converter.convert(&["42"]).unwrap();
        assert_eq!(value, 42_i64);
    }

    #[test]
    fn empty_array_returns_error() {
        // C 类（错误路径）：对标 Spring 空数组异常
        let converter = ArrayToObjectConverter;
        let result: Result<i64, _> = converter.convert(&Vec::<String>::new());
        let err = result.unwrap_err();
        assert!(err.reason.contains("空数组"));
    }
}
