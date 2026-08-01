//! 数字 → 字符转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.NumberToCharacterConverter`。

use crate::convert::{ConversionError, Converter};

/// 数字 → 字符转换器。
///
/// 对应 Java: org.springframework.core.convert.support.NumberToCharacterConverter
///
/// Spring 语义：数值按码位转换为字符（越界报错——对标
/// `ConversionFailedException`）。
pub struct NumberToCharacterConverter;

impl Converter<u32, char> for NumberToCharacterConverter {
    fn convert(&self, source: u32) -> Result<char, ConversionError> {
        char::from_u32(source).ok_or_else(|| ConversionError {
            value: source.to_string(),
            target_type: "char",
            reason: "码位超出 Unicode 标量值范围".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_code_point() {
        // A 类（合同对齐）：对标 Spring 数值 → 字符
        let converter = NumberToCharacterConverter;
        assert_eq!(converter.convert(65).unwrap(), 'A');
        assert_eq!(converter.convert(0x4E2D).unwrap(), '中');
    }

    #[test]
    fn out_of_range_returns_error() {
        // C 类（错误路径）：对标 Spring 越界异常
        let converter = NumberToCharacterConverter;
        assert!(converter.convert(0x0011_0000).is_err());
        assert!(converter.convert(0xD800).is_err()); // 代理区
    }
}
