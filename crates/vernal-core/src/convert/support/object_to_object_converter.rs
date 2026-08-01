//! 对象 → 对象转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.ObjectToObjectConverter`。

use crate::convert::{ConversionError, Convertible, Converter};

/// 对象 → 对象转换器。
///
/// 对应 Java: org.springframework.core.convert.support.ObjectToObjectConverter
///
/// Spring 语义：通过构造函数/静态工厂把源对象转换为目标类型；
/// Rust 中以 [`Convertible`]（`from_str_value`）表达同一转换路径。
pub struct ObjectToObjectConverter;

impl<T: Convertible> Converter<&str, T> for ObjectToObjectConverter {
    fn convert(&self, source: &str) -> Result<T, ConversionError> {
        T::from_str_value(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_via_convertible() {
        // A 类（合同对齐）：对标 Spring 工厂/构造器转换
        let converter = ObjectToObjectConverter;
        let value: u64 = converter.convert("42").unwrap();
        assert_eq!(value, 42_u64);
    }

    #[test]
    fn invalid_input_returns_error() {
        // C 类（错误路径）：对标 Spring `ConversionFailedException`
        let converter = ObjectToObjectConverter;
        let result: Result<u64, _> = converter.convert("nope");
        assert!(result.is_err());
    }
}
