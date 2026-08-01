//! 字符 → 数字转换器工厂。
//!
//! 对标 Spring `org.springframework.core.convert.support.CharacterToNumberFactory`。

use crate::convert::{ConversionError, Converter, ConverterFactory};

/// 字符 → 数字转换器工厂。
///
/// 对应 Java: org.springframework.core.convert.support.CharacterToNumberFactory
///
/// Spring 语义：`char` 的码位值（对标 `(short) charValue` 的数值解释）。
pub struct CharacterToNumberFactory;

/// 字符数值目标（Rust 中由整型解析承担）。
pub trait CharNumberTarget: Sized {
    /// 从字符码位解析为目标数值。
    fn from_char(c: char) -> Result<Self, ConversionError>;
}

macro_rules! impl_char_number {
    ($ty:ty) => {
        impl CharNumberTarget for $ty {
            fn from_char(c: char) -> Result<Self, ConversionError> {
                Ok(c as $ty)
            }
        }
    };
}

impl_char_number!(u8);
impl_char_number!(u16);
impl_char_number!(u32);
impl_char_number!(u64);
impl_char_number!(i32);
impl_char_number!(i64);

impl CharacterToNumberFactory {
    /// 把字符码位转换为目标数字类型。
    pub fn convert<T: CharNumberTarget>(&self, source: char) -> Result<T, ConversionError> {
        T::from_char(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_char_to_number() {
        // A 类（合同对齐）：对标 Spring char → Number（码位）
        let factory = CharacterToNumberFactory;
        assert_eq!(factory.convert::<u32>('A').unwrap(), 65_u32);
    }

    #[test]
    fn converts_unicode_char() {
        // B 类（边界行为）：Unicode 码位
        let factory = CharacterToNumberFactory;
        assert_eq!(factory.convert::<u32>('中').unwrap(), 0x4E2D);
    }
}
