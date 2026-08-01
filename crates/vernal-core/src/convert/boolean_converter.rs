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
        assert!(bool::from_str_value("true").unwrap());
        assert!(bool::from_str_value("1").unwrap());
        assert!(bool::from_str_value("yes").unwrap());
        assert!(bool::from_str_value("on").unwrap());
        assert!(bool::from_str_value("TRUE").unwrap());
        assert!(bool::from_str_value("Yes").unwrap());
        assert!(bool::from_str_value("ON").unwrap());
    }

    #[test]
    fn false_values() {
        assert!(!bool::from_str_value("false").unwrap());
        assert!(!bool::from_str_value("0").unwrap());
        assert!(!bool::from_str_value("no").unwrap());
        assert!(!bool::from_str_value("off").unwrap());
        assert!(!bool::from_str_value("FALSE").unwrap());
        assert!(!bool::from_str_value("No").unwrap());
        assert!(!bool::from_str_value("OFF").unwrap());
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
