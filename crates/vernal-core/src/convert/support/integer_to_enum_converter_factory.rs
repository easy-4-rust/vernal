//! 整数 → 枚举转换器工厂。
//!
//! 对标 Spring `org.springframework.core.convert.support.IntegerToEnumConverterFactory`。

use crate::convert::{ConversionError, Converter, ConverterFactory};

/// 整数 → 枚举转换器工厂。
///
/// 对应 Java: org.springframework.core.convert.support.IntegerToEnumConverterFactory
///
/// Spring 语义：按 `Enum.ordinal()` 反查变体（越界报错）。
pub struct IntegerToEnumConverterFactory;

/// 可由序号反查的枚举抽象（对标 Java `Enum.values()[ordinal]`）。
pub trait FromOrdinal: Sized {
    /// 由变体序号反查（越界返回 `None`）。
    fn from_ordinal(ordinal: u32) -> Option<Self>;
}

impl IntegerToEnumConverterFactory {
    /// 把变体序号转换为枚举值。
    ///
    /// # 错误
    ///
    /// 序号越界时返回 [`ConversionError`]。
    pub fn convert<T: FromOrdinal>(&self, source: u32) -> Result<T, ConversionError> {
        T::from_ordinal(source).ok_or_else(|| ConversionError {
            value: source.to_string(),
            target_type: std::any::type_name::<T>(),
            reason: "序号超出枚举变体范围".to_string(),
        })
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

    impl FromOrdinal for Color {
        fn from_ordinal(ordinal: u32) -> Option<Self> {
            match ordinal {
                0 => Some(Color::Red),
                1 => Some(Color::Green),
                2 => Some(Color::Blue),
                _ => None,
            }
        }
    }

    #[test]
    fn converts_ordinal_to_variant() {
        // A 类（合同对齐）：对标 Spring 序号 → 枚举
        let factory = IntegerToEnumConverterFactory;
        assert_eq!(factory.convert::<Color>(2).unwrap(), Color::Blue);
    }

    #[test]
    fn out_of_range_returns_error() {
        // C 类（错误路径）
        let factory = IntegerToEnumConverterFactory;
        assert!(factory.convert::<Color>(9).is_err());
    }
}
