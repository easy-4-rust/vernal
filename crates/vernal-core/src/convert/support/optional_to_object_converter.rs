//! Optional → 对象转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.OptionalToObjectConverter`。

use crate::convert::{ConversionError, Convertible, Converter};

/// `Option` → 对象转换器。
///
/// 对应 Java: org.springframework.core.convert.support.OptionalToObjectConverter
///
/// Spring 语义：`Optional` 展开为值（`Optional.empty` 转换失败——对标 Spring
/// 对 null 的 `ConversionFailedException`）。
pub struct OptionalToObjectConverter;

impl<T: Convertible> Converter<Option<&str>, T> for OptionalToObjectConverter {
    fn convert(&self, source: Option<&str>) -> Result<T, ConversionError> {
        let Some(value) = source else {
            return Err(ConversionError {
                value: String::new(),
                target_type: std::any::type_name::<T>(),
                reason: "Option 为空,无法转换为目标类型".to_string(),
            });
        };
        T::from_str_value(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unwraps_some_value() {
        // A 类（合同对齐）：对标 Spring `Optional.get()`
        let converter = OptionalToObjectConverter;
        let value: i32 = converter.convert(Some("10")).unwrap();
        assert_eq!(value, 10_i32);
    }

    #[test]
    fn none_returns_error() {
        // C 类（错误路径）：对标 Spring 空 Optional 异常
        let converter = OptionalToObjectConverter;
        let result: Result<i32, _> = converter.convert(None);
        assert!(result.is_err());
    }
}
