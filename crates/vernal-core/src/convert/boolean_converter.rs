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
