//! 字符串 → 枚举转换器工厂。
//!
//! 对标 Spring `org.springframework.core.convert.support.StringToEnumConverterFactory`。

use crate::convert::{ConversionError, Convertible, Converter, ConverterFactory};

/// 字符串 → 枚举转换器工厂。
///
/// 对应 Java: org.springframework.core.convert.support.StringToEnumConverterFactory
///
/// Spring 为每个枚举类型生成 `String → Enum` 转换器；Rust 以 `FromStr + Convertible`
/// 表达枚举转换语义（对标 `Enum.valueOf` + 大小写容错）。
pub struct StringToEnumConverterFactory;

impl<'a> ConverterFactory<&'a str> for StringToEnumConverterFactory {
    fn get_converter<T: Convertible>(&self) -> impl Converter<&'a str, T> {
        EnumFromStringConverter(std::marker::PhantomData)
    }
}

/// 内部转换器：委托 `Convertible::from_str_value`。
struct EnumFromStringConverter<T>(std::marker::PhantomData<T>);

impl<T: Convertible> Converter<&str, T> for EnumFromStringConverter<T> {
    fn convert(&self, source: &str) -> Result<T, ConversionError> {
        T::from_str_value(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    enum Color {
        Red,
        Green,
        Blue,
    }

    impl std::str::FromStr for Color {
        type Err = String;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_lowercase().as_str() {
                "red" => Ok(Color::Red),
                "green" => Ok(Color::Green),
                "blue" => Ok(Color::Blue),
                _ => Err(format!("unknown color: {s}")),
            }
        }
    }

    impl Convertible for Color {
        fn from_str_value(value: &str) -> Result<Self, ConversionError> {
            value.parse().map_err(|e| ConversionError {
                value: value.to_string(),
                target_type: "Color",
                reason: e,
            })
        }
    }

    #[test]
    fn factory_generates_enum_converter() {
        // A 类（合同对齐）：对标 Spring `getConverter(Color.class)`
        let factory = StringToEnumConverterFactory;
        let converter = factory.get_converter::<Color>();
        assert_eq!(converter.convert("red").unwrap(), Color::Red);
    }

    #[test]
    fn unknown_variant_returns_error() {
        // C 类（错误路径）：对标 Spring `ConversionFailedException`
        let factory = StringToEnumConverterFactory;
        let converter = factory.get_converter::<Color>();
        assert!(converter.convert("purple").is_err());
    }
}
