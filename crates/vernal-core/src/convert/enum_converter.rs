//! 枚举转换器（字符串 ↔ 枚举）。
//!
//! 提供 `convert_enum` 辅助函数，用于将字符串转换为实现了 `FromStr` 的枚举类型。

use super::ConversionError;

/// 将字符串转换为实现了 `FromStr` 的枚举类型。
///
/// 用于配置属性绑定中的枚举字段。
///
/// # 示例
///
/// ```rust
/// use vernal_core::convert::convert_enum;
///
/// #[derive(Debug, PartialEq)]
/// enum Mode { Production, Development }
///
/// impl std::str::FromStr for Mode {
///     type Err = String;
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         match s.to_lowercase().as_str() {
///             "production" | "prod" => Ok(Mode::Production),
///             "development" | "dev" => Ok(Mode::Development),
///             _ => Err(format!("未知模式: {}", s)),
///         }
///     }
/// }
///
/// let mode: Mode = convert_enum("production").unwrap();
/// assert_eq!(mode, Mode::Production);
/// ```
pub fn convert_enum<T: std::str::FromStr>(value: &str) -> Result<T, ConversionError>
where
    T::Err: std::fmt::Display,
{
    value.parse::<T>().map_err(|e| ConversionError {
        value: value.to_string(),
        target_type: std::any::type_name::<T>(),
        reason: e.to_string(),
    })
}
