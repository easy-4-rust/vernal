//! 字符串转换器（String → 其他类型）。

use super::{ConversionError, Convertible};

/// 字符串转换器。
///
/// 处理 `String` 到其他类型的转换。
pub struct StringConverter;

/// `String` 实现 `Convertible`（直接返回自身）。
impl Convertible for String {
    fn from_str_value(value: &str) -> Result<Self, ConversionError> {
        Ok(value.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_string_to_string() {
        assert_eq!(String::from_str_value("hello").unwrap(), "hello");
    }

    #[test]
    fn empty_string_returns_empty() {
        assert_eq!(String::from_str_value("").unwrap(), "");
    }

    #[test]
    fn preserves_whitespace() {
        assert_eq!(String::from_str_value("  hello  ").unwrap(), "  hello  ");
    }

    #[test]
    fn unicode_roundtrip() {
        assert_eq!(String::from_str_value("你好世界").unwrap(), "你好世界");
    }
}
