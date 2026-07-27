//! 布尔转换器（字符串 ↔ bool）。

use super::{ConversionError, Convertible};

/// 布尔转换器。
///
/// 支持的字符串值：
/// - `"true"`, `"1"`, `"yes"`, `"on"` → `true`
/// - `"false"`, `"0"`, `"no"`, `"off"` → `false`
/// - 大小写不敏感
pub struct BooleanConverter;

impl Convertible for bool {
    fn from_str_value(value: &str) -> Result<Self, ConversionError> {
        match value.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok(true),
            "false" | "0" | "no" | "off" => Ok(false),
            _ => Err(ConversionError {
                value: value.to_string(),
                target_type: "bool",
                reason: "期望 true/false/1/0/yes/no/on/off".to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn true_values() {
        assert_eq!(bool::from_str_value("true").unwrap(), true);
        assert_eq!(bool::from_str_value("1").unwrap(), true);
        assert_eq!(bool::from_str_value("yes").unwrap(), true);
        assert_eq!(bool::from_str_value("on").unwrap(), true);
        assert_eq!(bool::from_str_value("TRUE").unwrap(), true);
        assert_eq!(bool::from_str_value("Yes").unwrap(), true);
        assert_eq!(bool::from_str_value("ON").unwrap(), true);
    }

    #[test]
    fn false_values() {
        assert_eq!(bool::from_str_value("false").unwrap(), false);
        assert_eq!(bool::from_str_value("0").unwrap(), false);
        assert_eq!(bool::from_str_value("no").unwrap(), false);
        assert_eq!(bool::from_str_value("off").unwrap(), false);
        assert_eq!(bool::from_str_value("FALSE").unwrap(), false);
        assert_eq!(bool::from_str_value("No").unwrap(), false);
        assert_eq!(bool::from_str_value("OFF").unwrap(), false);
    }

    #[test]
    fn invalid_returns_error() {
        let err = bool::from_str_value("invalid").unwrap_err();
        assert_eq!(err.target_type, "bool");
    }

    #[test]
    fn empty_returns_error() {
        let err = bool::from_str_value("").unwrap_err();
        assert_eq!(err.target_type, "bool");
    }
}
