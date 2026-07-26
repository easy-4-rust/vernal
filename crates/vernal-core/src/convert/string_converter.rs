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
