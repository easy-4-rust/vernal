//! 字符串 → 数字转换器工厂。
//!
//! 对标 Spring `org.springframework.core.convert.support.StringToNumberConverterFactory`。

use crate::convert::{ConversionError, Converter, ConverterFactory, Convertible};

/// 字符串 → 数字转换器工厂。
///
/// 对应 Java: org.springframework.core.convert.support.StringToNumberConverterFactory
///
/// Spring 中该工厂为每个 `Number` 子类生成转换器；Rust 以 [`Convertible`] 表达
/// "可被字符串解析的数字目标类型族"（对标 Spring `Number` 层次）。
pub struct StringToNumberConverterFactory;

impl<'a> ConverterFactory<&'a str> for StringToNumberConverterFactory {
    fn get_converter<T: Convertible>(&self) -> impl Converter<&'a str, T> {
        NumberFromStringConverter(std::marker::PhantomData)
    }
}

/// 内部转换器：委托 `Convertible::from_str_value`。
struct NumberFromStringConverter<T>(std::marker::PhantomData<T>);

impl<T: Convertible> Converter<&str, T> for NumberFromStringConverter<T> {
    fn convert(&self, source: &str) -> Result<T, ConversionError> {
        T::from_str_value(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_generates_integer_converter() {
        // A 类（合同对齐）：对标 Spring `getConverter(Integer.class)`
        let factory = StringToNumberConverterFactory;
        let converter = factory.get_converter::<i64>();
        assert_eq!(converter.convert("42").unwrap(), 42_i64);
    }

    #[test]
    fn factory_generates_float_converter() {
        let factory = StringToNumberConverterFactory;
        let converter = factory.get_converter::<f64>();
        let value = converter.convert("3.25").unwrap();
        assert!((value - 3.25).abs() < 1e-12);
    }

    #[test]
    fn invalid_number_returns_error() {
        // C 类（错误路径）：对标 Spring `ConversionFailedException`
        let factory = StringToNumberConverterFactory;
        let converter = factory.get_converter::<u32>();
        assert!(converter.convert("not-a-number").is_err());
    }
}
