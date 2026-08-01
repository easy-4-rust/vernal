//! 字符串 → 正则模式转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.StringToPatternConverter`
//! （Java 中以 `Pattern.compile` 表达）。仅在 feature `convert-regex` 下编译。

use crate::convert::{ConversionError, Convertible, Converter};

/// 字符串 → 正则模式转换器。
///
/// 对应 Java: org.springframework.core.convert.support.StringToPatternConverter
pub struct StringToPatternConverter;

impl Converter<&str, regex::Regex> for StringToPatternConverter {
    fn convert(&self, source: &str) -> Result<regex::Regex, ConversionError> {
        regex::Regex::from_str_value(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_valid_pattern() {
        // A 类（合同对齐）：对标 Spring `Pattern.compile`
        let converter = StringToPatternConverter;
        let pattern = converter.convert(r"^\d+$").unwrap();
        assert!(pattern.is_match("123"));
        assert!(!pattern.is_match("12a"));
    }

    #[test]
    fn invalid_pattern_returns_error() {
        // C 类（错误路径）：对标 Spring `PatternSyntaxException`
        let converter = StringToPatternConverter;
        assert!(converter.convert("([").is_err());
    }
}
