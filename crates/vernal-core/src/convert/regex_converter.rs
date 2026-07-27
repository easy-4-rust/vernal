//! 正则表达式转换器(feature = "convert-regex")。
//!
//! 对标 Spring 的 `StringToPatternConverter` 与 `StringToRegexConverter`。
//!
//! # 启用方式
//!
//! ```toml
//! [dependencies]
//! vernal-core = { features = ["convert-regex"] }
//! ```

use super::{ConversionError, Convertible};

/// 正则表达式转换器。
///
/// 对标 Spring `StringToPatternConverter`(Pattern) 与 `StringToRegexConverter`。
pub struct RegexConverter;

impl Convertible for regex::Regex {
    fn from_str_value(value: &str) -> Result<Self, ConversionError> {
        regex::Regex::new(value).map_err(|e| ConversionError {
            value: value.to_string(),
            target_type: "Regex",
            reason: format!("正则表达式解析失败: {e}"),
        })
    }
}

impl Convertible for regex::RegexBuilder {
    fn from_str_value(value: &str) -> Result<Self, ConversionError> {
        // RegexBuilder::new 没有 public constructor for clone,这里直接返回 new
        // 注意:RegexBuilder 不可 Clone,所以业务需要自己处理
        Ok(regex::RegexBuilder::new(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_pattern() {
        let re = regex::Regex::from_str_value(r"\d+").unwrap();
        assert!(re.is_match("123"));
        assert!(!re.is_match("abc"));
    }

    #[test]
    fn parses_word_boundaries() {
        let re = regex::Regex::from_str_value(r"\bhello\b").unwrap();
        assert!(re.is_match("hello world"));
        assert!(!re.is_match("helloworld"));
    }

    #[test]
    fn parses_empty_pattern_as_match_nothing_or_everything() {
        // 注意:Rust regex 接受空模式(对标 Java Pattern.compile("") 不抛错)
        // 这里验证空模式确实可解析
        let re = regex::Regex::from_str_value("").unwrap();
        // 空模式匹配空字符串,所以任何字符串都能匹配(因为包含空字符串)
        assert!(re.is_match(""));
        assert!(re.is_match("abc"));
    }

    #[test]
    fn rejects_invalid_pattern() {
        // 不匹配的括号
        let err = regex::Regex::from_str_value("(unclosed").unwrap_err();
        assert_eq!(err.target_type, "Regex");
        assert!(err.reason.contains("解析失败"));
    }

    #[test]
    fn parses_complex_pattern() {
        let re = regex::Regex::from_str_value(r"^[a-zA-Z0-9]+@[a-zA-Z0-9]+\.[a-z]+$").unwrap();
        assert!(re.is_match("user@example.com"));
        assert!(!re.is_match("invalid"));
    }

    #[test]
    fn regex_via_conversion_service() {
        let re: regex::Regex =
            super::super::ConversionService::convert(r"\d{4}-\d{2}-\d{2}").unwrap();
        assert!(re.is_match("2026-07-27"));
    }
}
