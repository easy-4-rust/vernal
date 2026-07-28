//! 可转换的目标类型 trait。
//!
//! 对标 Spring `org.springframework.core.convert.converter.Converter`。

use super::conversion_error::ConversionError;

/// 可转换的目标类型 trait。
///
/// 实现此 trait 的类型可以从字符串值转换而来。
/// 对应 Java: `Converter<S, T>` 接口。
pub trait Convertible: Sized {
    /// 从字符串值转换。
    ///
    /// 对应 Java: `Converter#convert(S)`
    ///
    /// # Errors
    ///
    /// 当输入值无法解析为目标类型时返回 [`ConversionError`]。
    fn from_str_value(value: &str) -> Result<Self, ConversionError>;
}
