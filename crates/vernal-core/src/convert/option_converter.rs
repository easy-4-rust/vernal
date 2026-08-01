//! 可选值转换器（处理 `Option<T>`）。

use super::{ConversionError, Convertible};

/// 可选值转换器。
///
/// 处理 `Option<T>` 类型的转换：
/// - 空字符串 → `None`
/// - 非空字符串 → `Some(T)`
pub struct OptionConverter;

/// 为 `Option<T>` 实现 `Convertible`。
///
/// 空字符串被转换为 `None`，非空字符串被转换为 `Some(T)`。
impl<T: Convertible> Convertible for Option<T> {
    fn from_str_value(value: &str) -> Result<Self, ConversionError> {
        if value.is_empty() {
            Ok(None)
        } else {
            T::from_str_value(value).map(Some)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_returns_none() {
        let result = Option::<String>::from_str_value("").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn non_empty_string_returns_some() {
        let result = Option::<String>::from_str_value("hello").unwrap();
        assert_eq!(result, Some("hello".to_string()));
    }

    #[test]
    fn option_bool_empty_returns_none() {
        let result = Option::<bool>::from_str_value("").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn option_bool_non_empty_returns_some() {
        let result = Option::<bool>::from_str_value("true").unwrap();
        assert_eq!(result, Some(true));
    }

    #[test]
    fn option_i32_empty_returns_none() {
        let result = Option::<i32>::from_str_value("").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn option_i32_valid_returns_some() {
        let result = Option::<i32>::from_str_value("42").unwrap();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn option_i32_invalid_returns_error() {
        let result = Option::<i32>::from_str_value("abc");
        assert!(result.is_err());
    }
}
