//! 可选值转换器（处理 Option<T>）。

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
