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
