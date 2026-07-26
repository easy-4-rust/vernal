//! 转换器 trait。

use super::ConversionError;

/// 类型转换器 trait。
///
/// 对标 Spring 的 `org.springframework.core.convert.converter.Converter`。
pub trait Converter<S, T> {
    /// 将源类型转换为目标类型。
    fn convert(&self, source: S) -> Result<T, ConversionError>;
}
