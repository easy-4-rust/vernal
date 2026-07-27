//! 数字转换器（字符串 ↔ 数字）。

use super::{ConversionError, Convertible};

/// 数字转换器。
///
/// 处理字符串到数字类型的转换。
pub struct NumberConverter;

/// 为所有数字类型实现 `Convertible`。
macro_rules! impl_convertible_for_number {
    ($ty:ty, $name:expr) => {
        impl Convertible for $ty {
            fn from_str_value(value: &str) -> Result<Self, ConversionError> {
                value.parse().map_err(|_| ConversionError {
                    value: value.to_string(),
                    target_type: $name,
                    reason: "数字格式无效".to_string(),
                })
            }
        }
    };
}

impl_convertible_for_number!(i8, "i8");
impl_convertible_for_number!(i16, "i16");
impl_convertible_for_number!(i32, "i32");
impl_convertible_for_number!(i64, "i64");
impl_convertible_for_number!(i128, "i128");
impl_convertible_for_number!(isize, "isize");
impl_convertible_for_number!(u8, "u8");
impl_convertible_for_number!(u16, "u16");
impl_convertible_for_number!(u32, "u32");
impl_convertible_for_number!(u64, "u64");
impl_convertible_for_number!(u128, "u128");
impl_convertible_for_number!(usize, "usize");
impl_convertible_for_number!(f32, "f32");
impl_convertible_for_number!(f64, "f64");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i32_valid() {
        assert_eq!(i32::from_str_value("42").unwrap(), 42);
        assert_eq!(i32::from_str_value("-1").unwrap(), -1);
        assert_eq!(i32::from_str_value("0").unwrap(), 0);
    }

    #[test]
    fn i32_invalid() {
        let err = i32::from_str_value("abc").unwrap_err();
        assert_eq!(err.target_type, "i32");
    }

    #[test]
    fn u64_valid() {
        assert_eq!(u64::from_str_value("123456789").unwrap(), 123456789u64);
    }

    #[test]
    fn u64_overflow() {
        let err = u64::from_str_value("-1").unwrap_err();
        assert_eq!(err.target_type, "u64");
    }

    #[test]
    fn f64_valid() {
        assert!((f64::from_str_value("3.14").unwrap() - 3.14).abs() < 1e-10);
    }

    #[test]
    fn f32_valid() {
        assert!((f32::from_str_value("2.5").unwrap() - 2.5).abs() < 1e-6);
    }

    #[test]
    fn i8_boundary() {
        assert_eq!(i8::from_str_value("127").unwrap(), 127);
        assert_eq!(i8::from_str_value("-128").unwrap(), -128);
        assert!(i8::from_str_value("128").is_err());
    }

    #[test]
    fn usize_valid() {
        assert_eq!(usize::from_str_value("100").unwrap(), 100usize);
    }

    #[test]
    fn empty_string_fails() {
        assert!(i32::from_str_value("").is_err());
        assert!(f64::from_str_value("").is_err());
    }
}
