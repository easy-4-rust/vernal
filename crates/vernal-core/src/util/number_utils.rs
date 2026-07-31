//! 数字工具。
//!
//! 对标 Spring `org.springframework.util.NumberUtils`。
//!
//! Spring 的多数方法依赖 Java `java.lang.Number` 抽象与 `Class<T>` 反射,
//! 在 Rust 中用泛型与 trait 替代更自然。

/// 数字工具。
///
/// 对标 Spring `NumberUtils`。
pub struct NumberUtils;

impl NumberUtils {
    /// 容错解析字符串为 `i64`。
    ///
    /// 对标 Spring `NumberUtils.parseNumber(String, Class)`(简化版)。
    ///
    /// 与 `str::parse` 不同:本方法接受十六进制(`0x...`)与负号。
    #[must_use]
    pub fn parse_i64(s: &str) -> Option<i64> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }
        // 支持十六进制
        if let Some(hex) = trimmed
            .strip_prefix("0x")
            .or_else(|| trimmed.strip_prefix("0X"))
        {
            return i64::from_str_radix(hex, 16).ok();
        }
        // 支持八进制
        if let Some(oct) = trimmed
            .strip_prefix("0o")
            .or_else(|| trimmed.strip_prefix("0O"))
        {
            return i64::from_str_radix(oct, 8).ok();
        }
        // 支持二进制
        if let Some(bin) = trimmed
            .strip_prefix("0b")
            .or_else(|| trimmed.strip_prefix("0B"))
        {
            return i64::from_str_radix(bin, 2).ok();
        }
        trimmed.parse().ok()
    }

    /// 容错解析字符串为 `f64`。
    ///
    /// 对标 Spring `NumberUtils.parseNumber(String, Double.class)`。
    #[must_use]
    pub fn parse_f64(s: &str) -> Option<f64> {
        s.trim().parse().ok()
    }

    /// 容错解析字符串为 `i32`。
    #[must_use]
    pub fn parse_i32(s: &str) -> Option<i32> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Some(hex) = trimmed
            .strip_prefix("0x")
            .or_else(|| trimmed.strip_prefix("0X"))
        {
            return i32::from_str_radix(hex, 16).ok();
        }
        trimmed.parse().ok()
    }

    /// 把 i64 转换为目标数字类型(对标 Spring `convertNumberToTargetClass`)。
    ///
    /// 在 Rust 中用 `TryFrom` trait 替代,本方法仅为兼容性提供。
    #[must_use]
    pub fn convert_to_target<T>(value: i64) -> Option<T>
    where
        T: TryFrom<i64>,
    {
        T::try_from(value).ok()
    }

    /// 检查字符串是否是合法的十进制整数。
    ///
    /// 对标 Spring `NumberUtils.isDigits(String)`(仅数字字符)。
    #[must_use]
    pub fn is_digits(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
    }

    /// 检查字符串是否是合法的数字(可含负号、小数点)。
    #[must_use]
    pub fn is_number(s: &str) -> bool {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return false;
        }
        trimmed.parse::<f64>().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_i64_decimal() {
        assert_eq!(NumberUtils::parse_i64("42"), Some(42));
        assert_eq!(NumberUtils::parse_i64("-100"), Some(-100));
        assert_eq!(NumberUtils::parse_i64("  123  "), Some(123));
    }

    #[test]
    fn parse_i64_hex() {
        assert_eq!(NumberUtils::parse_i64("0xff"), Some(255));
        assert_eq!(NumberUtils::parse_i64("0X10"), Some(16));
    }

    #[test]
    fn parse_i64_octal() {
        assert_eq!(NumberUtils::parse_i64("0o17"), Some(15));
    }

    #[test]
    fn parse_i64_binary() {
        assert_eq!(NumberUtils::parse_i64("0b1010"), Some(10));
    }

    #[test]
    fn parse_i64_invalid() {
        assert_eq!(NumberUtils::parse_i64(""), None);
        assert_eq!(NumberUtils::parse_i64("abc"), None);
        assert_eq!(NumberUtils::parse_i64("  "), None);
    }

    #[test]
    fn parse_f64_basic() {
        assert_eq!(NumberUtils::parse_f64("3.14"), Some(3.14));
        assert_eq!(NumberUtils::parse_f64("-0.5"), Some(-0.5));
        assert_eq!(NumberUtils::parse_f64("1e10"), Some(1e10));
    }

    #[test]
    fn parse_f64_invalid() {
        assert_eq!(NumberUtils::parse_f64("abc"), None);
    }

    #[test]
    fn parse_i32_basic() {
        assert_eq!(NumberUtils::parse_i32("42"), Some(42));
        assert_eq!(NumberUtils::parse_i32("0xff"), Some(255));
    }

    #[test]
    fn convert_to_target_works() {
        let v: u8 = NumberUtils::convert_to_target(42).unwrap();
        assert_eq!(v, 42u8);
    }

    #[test]
    fn convert_to_target_overflow_returns_none() {
        let v: Option<u8> = NumberUtils::convert_to_target(300);
        assert!(v.is_none());
    }

    #[test]
    fn is_digits_basic() {
        assert!(NumberUtils::is_digits("12345"));
        assert!(NumberUtils::is_digits("0"));
        assert!(!NumberUtils::is_digits(""));
        assert!(!NumberUtils::is_digits("12.34"));
        assert!(!NumberUtils::is_digits("-12"));
        assert!(!NumberUtils::is_digits("abc"));
    }

    #[test]
    fn is_number_basic() {
        assert!(NumberUtils::is_number("123"));
        assert!(NumberUtils::is_number("-3.14"));
        assert!(NumberUtils::is_number("1e10"));
        assert!(!NumberUtils::is_number(""));
        assert!(!NumberUtils::is_number("abc"));
        assert!(!NumberUtils::is_number("12.34.56"));
    }

    #[test]
    fn parse_i32_empty_and_whitespace_returns_none() {
        // 对标 Spring: NumberUtils.parseNumber("", Integer.class) 返回 null
        // 覆盖行 62: parse_i32 空字符串 → return None
        assert_eq!(NumberUtils::parse_i32(""), None);
        assert_eq!(NumberUtils::parse_i32("   "), None);
    }
}
