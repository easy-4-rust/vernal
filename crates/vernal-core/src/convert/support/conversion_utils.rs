//! 转换工具。
//!
//! 对标 Spring `org.springframework.core.convert.support.ConversionUtils`。

use std::any::TypeId;

use crate::convert::{ConversionError, Convertible, Converter};

/// 转换工具（静态辅助函数）。
///
/// 对应 Java: org.springframework.core.convert.support.ConversionUtils
pub struct ConversionUtils;

impl ConversionUtils {
    /// 调用单个转换器（对标 Spring `invokeConverter`）。
    ///
    /// # 错误
    ///
    /// 转换器返回错误时原样传播。
    pub fn invoke_converter<T, U>(
        converter: &dyn Converter<T, U>,
        source: T,
    ) -> Result<U, ConversionError> {
        converter.convert(source)
    }

    /// 判定类型对是否可转换（对标 Spring `canConvert` 的静态形态）。
    #[must_use]
    pub fn can_convert<T: Convertible + 'static>() -> bool {
        let _ = TypeId::of::<T>();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert::support::StringToBooleanConverter;

    #[test]
    fn invokes_converter() {
        // A 类（合同对齐）：对标 Spring `invokeConverter`
        let converter = StringToBooleanConverter;
        assert!(ConversionUtils::invoke_converter(&converter, "yes").unwrap());
    }

    #[test]
    fn converter_error_propagates() {
        // C 类（错误路径）
        let converter = StringToBooleanConverter;
        assert!(ConversionUtils::invoke_converter(&converter, "nope").is_err());
    }

    #[test]
    fn can_convert_known_types() {
        // B 类（边界行为）
        assert!(ConversionUtils::can_convert::<u64>());
        assert!(ConversionUtils::can_convert::<bool>());
    }
}
