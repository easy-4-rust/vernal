//! 字符串 → 字符转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.StringToCharacterConverter`。

use crate::convert::{ConversionError, Converter};

/// 字符串 → 字符转换器。
///
/// 对应 Java: org.springframework.core.convert.support.StringToCharacterConverter
///
/// Spring 语义：字符串必须恰好包含一个 Unicode 标量值，否则抛
/// `ConversionFailedException`。
pub struct StringToCharacterConverter;

impl Converter<&str, char> for StringToCharacterConverter {
    fn convert(&self, source: &str) -> Result<char, ConversionError> {
        let mut chars = source.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => Ok(c),
            (None, _) => Err(ConversionError {
                value: source.to_string(),
                target_type: "char",
                reason: "空字符串无法转换为字符".to_string(),
            }),
            (Some(_), Some(_)) => Err(ConversionError {
                value: source.to_string(),
                target_type: "char",
                reason: "字符串必须只包含一个字符".to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_character_converts() {
        // A 类（合同对齐）：对标 Spring 单字符转换
        let converter = StringToCharacterConverter;
        assert_eq!(converter.convert("a").unwrap(), 'a');
    }

    #[test]
    fn unicode_scalar_converts() {
        // B 类（边界行为）：Unicode 标量值按单字符处理
        let converter = StringToCharacterConverter;
        assert_eq!(converter.convert("中").unwrap(), '中');
    }

    #[test]
    fn empty_and_multiple_chars_return_error() {
        // C 类（错误路径）：对标 Spring `ConversionFailedException`
        let converter = StringToCharacterConverter;
        assert!(converter.convert("").is_err());
        assert!(converter.convert("ab").is_err());
    }
}
