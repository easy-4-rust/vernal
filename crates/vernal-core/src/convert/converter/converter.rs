//! 转换器 trait。

use crate::convert::ConversionError;

/// 类型转换器 trait。
///
/// 对应 Java: org.springframework.core.convert.converter.Converter
pub trait Converter<S, T> {
    /// 将源类型转换为目标类型。
    fn convert(&self, source: S) -> Result<T, ConversionError>;
}
