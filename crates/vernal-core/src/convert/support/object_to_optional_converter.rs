//! 对象 → Optional 转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.ObjectToOptionalConverter`。

use crate::convert::{ConversionError, Convertible, Converter};

/// 对象 → `Option` 转换器。
///
/// 对应 Java: org.springframework.core.convert.support.ObjectToOptionalConverter
///
/// Spring 语义：把值包装为 `Optional`；Rust 中以 `Option` 表达。
pub struct ObjectToOptionalConverter;

impl<T: Convertible> Converter<&str, Option<T>> for ObjectToOptionalConverter {
    fn convert(&self, source: &str) -> Result<Option<T>, ConversionError> {
        T::from_str_value(source).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_value_into_option() {
        // A 类（合同对齐）：对标 Spring 值 → Optional
        let converter = ObjectToOptionalConverter;
        let value: Option<u8> = converter.convert("1").unwrap();
        assert_eq!(value, Some(1_u8));
    }

    #[test]
    fn invalid_value_returns_error() {
        // C 类（错误路径）
        let converter = ObjectToOptionalConverter;
        let result: Result<Option<i32>, _> = converter.convert("x");
        assert!(result.is_err());
    }
}
