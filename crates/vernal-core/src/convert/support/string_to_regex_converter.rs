//! 字符串 → 正则表达式转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.StringToRegexConverter`。
//! 仅在 feature `convert-regex` 下编译。

use crate::convert::{ConversionError, Convertible, Converter};

/// 字符串 → 正则表达式转换器。
///
/// 对应 Java: org.springframework.core.convert.support.StringToRegexConverter
pub struct StringToRegexConverter;

impl Converter<&str, regex::Regex> for StringToRegexConverter {
    fn convert(&self, source: &str) -> Result<regex::Regex, ConversionError> {
        regex::Regex::from_str_value(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_and_matches() {
        // A 类（合同对齐）：对标 Spring 正则转换
        let converter = StringToRegexConverter;
        let regex = converter.convert(r"colou?r").unwrap();
        assert!(regex.is_match("color"));
        assert!(regex.is_match("colour"));
    }

    #[test]
    fn invalid_regex_returns_error() {
        // C 类（错误路径）
        let converter = StringToRegexConverter;
        assert!(converter.convert("*bad").is_err());
    }
}
