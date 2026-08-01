//! 转换异常（转换体系基类）。
//!
//! 对标 Spring `org.springframework.core.convert.ConversionException`。

use crate::convert::ConversionError;

/// 转换异常。
///
/// 对应 Java: org.springframework.core.convert.ConversionException
///
/// vernal 的转换错误统一由 [`ConversionError`] 表达（对标 Spring
/// `ConversionException` 作为所有转换异常的基类）。
pub type ConversionException = ConversionError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alias_is_usable_as_error() {
        // D 类（重构安全）：别名保持错误语义
        fn assert_error<T: std::error::Error>() {}
        assert_error::<ConversionException>();
    }

    #[test]
    fn alias_round_trips_with_conversion_error() {
        let err: ConversionException = ConversionError {
            value: "x".to_string(),
            target_type: "i32",
            reason: "bad".to_string(),
        };
        assert_eq!(err.value, "x");
        assert!(err.to_string().contains("bad"));
    }
}
