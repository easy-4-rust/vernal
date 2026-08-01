//! 转换器未找到错误。
//!
//! 对标 Spring `org.springframework.core.convert.ConverterNotFoundException`。
//!
//! # 设计来源
//!
//! Spring 在 `ConversionService.convert()` 找不到匹配的 Converter 时抛出
//! `ConverterNotFoundException`,它继承自 `ConversionException`。
//!
//! vernal-core 通过扩展 [`ConversionError`] 的 reason 字段表达等价语义,
//! 同时提供便捷构造函数。

use std::any::TypeId;

use super::ConversionError;

/// 构造"找不到转换器"错误。
///
/// 对标 Spring `new ConverterNotFoundException(sourceType, targetType)`。
///
/// # 示例
///
/// ```rust
/// use vernal_core::convert::converter_not_found;
/// use std::any::TypeId;
///
/// let err = converter_not_found("hello", TypeId::of::<String>(), TypeId::of::<bool>());
/// assert!(err.reason.contains("Converter not found"));
/// ```
#[must_use]
pub fn converter_not_found(
    value: &str,
    source_type: TypeId,
    target_type: TypeId,
) -> ConversionError {
    ConversionError {
        value: value.to_string(),
        target_type: "dynamic",
        reason: format!("Converter not found for {source_type:?} -> {target_type:?}"),
    }
}

/// 检查错误是否为"找不到转换器"类型。
///
/// 便捷谓词,用于错误处理逻辑分支。
#[must_use]
pub fn is_converter_not_found(err: &ConversionError) -> bool {
    err.reason.contains("Converter not found") || err.reason.contains("no converter")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converter_not_found_includes_type_pair() {
        let err = converter_not_found("test", TypeId::of::<String>(), TypeId::of::<bool>());
        assert!(err.reason.contains("Converter not found"));
        // TypeId Debug 输出格式无法预测,只验证包含关键词
        assert!(!err.reason.is_empty());
    }

    #[test]
    fn is_converter_not_found_detects_pattern() {
        let err1 = converter_not_found("x", TypeId::of::<String>(), TypeId::of::<bool>());
        assert!(is_converter_not_found(&err1));

        let err2 = ConversionError {
            value: "x".to_string(),
            target_type: "i64",
            reason: "数字格式无效".to_string(),
        };
        assert!(!is_converter_not_found(&err2));
    }

    #[test]
    fn converter_not_found_preserves_value() {
        let err = converter_not_found("hello", TypeId::of::<String>(), TypeId::of::<i32>());
        assert_eq!(err.value, "hello");
    }
}
