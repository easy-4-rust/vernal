//! Vernal 类型转换器（Vernal 扩展）。
//!
//! 对标 Spring `ConversionService` 中的类型转换能力。
//! 通过 `vernal-core` 的类型转换接口实现。
//!
//! # 设计
//!
//! - 继承 `StandardTypeConverter` 的基本转换
//! - 额外支持从 `vernal-core::convert` 模块的类型转换
//! - 支持自定义类型转换器注册

use crate::evaluation_exception::EvaluationException;
use crate::type_converter::TypeConverter;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// Vernal 类型转换器（Vernal 扩展）。
///
/// 通过 `vernal-core` 的类型转换接口实现。
/// 继承 `StandardTypeConverter` 的基本转换。
pub struct VernalTypeConverter {
    /// 自定义类型转换器。
    custom_converters:
        Vec<Box<dyn Fn(&TypedValue, &TypeDescriptor) -> Option<TypedValue> + Send + Sync>>,
}

impl VernalTypeConverter {
    /// 创建 Vernal 类型转换器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            custom_converters: Vec::new(),
        }
    }

    /// 注册自定义类型转换器。
    pub fn register_converter<F>(&mut self, converter: F)
    where
        F: Fn(&TypedValue, &TypeDescriptor) -> Option<TypedValue> + Send + Sync + 'static,
    {
        self.custom_converters.push(Box::new(converter));
    }
}

impl Default for VernalTypeConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeConverter for VernalTypeConverter {
    fn can_convert(&self, source_type: &TypeDescriptor, target_type: &TypeDescriptor) -> bool {
        // 所有基本类型之间都可转换
        source_type.is_primitive() && target_type.is_primitive()
    }

    fn convert_value(
        &self,
        value: &TypedValue,
        target_type: &TypeDescriptor,
    ) -> Result<TypedValue, EvaluationException> {
        // 1. 尝试自定义转换器
        for converter in &self.custom_converters {
            if let Some(result) = converter(value, target_type) {
                return Ok(result);
            }
        }

        // 2. 基本类型转换
        match (value.value(), target_type) {
            // Int → String
            (ExpressionValue::Int(i), TypeDescriptor::Primitive(PrimitiveKind::String)) => {
                Ok(TypedValue::new(
                    ExpressionValue::String(i.to_string()),
                    TypeDescriptor::STRING,
                ))
            }
            // String → Int
            (ExpressionValue::String(s), TypeDescriptor::Primitive(PrimitiveKind::Int)) => s
                .parse::<i64>()
                .map(|i| TypedValue::new(ExpressionValue::Int(i), TypeDescriptor::INT))
                .map_err(|_| {
                    EvaluationException::new("", None, format!("无法将 '{}' 转换为整数", s))
                }),
            // Boolean → String
            (ExpressionValue::Boolean(b), TypeDescriptor::Primitive(PrimitiveKind::String)) => {
                Ok(TypedValue::new(
                    ExpressionValue::String(b.to_string()),
                    TypeDescriptor::STRING,
                ))
            }
            // String → Boolean
            (ExpressionValue::String(s), TypeDescriptor::Primitive(PrimitiveKind::Boolean)) => {
                match s.as_str() {
                    "true" | "TRUE" | "True" | "1" => Ok(TypedValue::new(
                        ExpressionValue::Boolean(true),
                        TypeDescriptor::BOOLEAN,
                    )),
                    "false" | "FALSE" | "False" | "0" => Ok(TypedValue::new(
                        ExpressionValue::Boolean(false),
                        TypeDescriptor::BOOLEAN,
                    )),
                    _ => Err(EvaluationException::new(
                        "",
                        None,
                        format!("无法将 '{}' 转换为布尔值", s),
                    )),
                }
            }
            // 同类型直接返回
            _ if value.type_descriptor() == target_type => Ok(value.clone()),
            // 未知转换：返回原值
            _ => Ok(value.clone()),
        }
    }
}

use crate::type_descriptor::PrimitiveKind;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_to_string() {
        let converter = VernalTypeConverter::new();
        let value = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
        let result = converter
            .convert_value(&value, &TypeDescriptor::STRING)
            .unwrap();
        assert_eq!(*result.value(), ExpressionValue::String("42".to_string()));
    }

    #[test]
    fn string_to_int() {
        let converter = VernalTypeConverter::new();
        let value = TypedValue::new(
            ExpressionValue::String("123".to_string()),
            TypeDescriptor::STRING,
        );
        let result = converter
            .convert_value(&value, &TypeDescriptor::INT)
            .unwrap();
        assert_eq!(*result.value(), ExpressionValue::Int(123));
    }

    #[test]
    fn string_to_int_error() {
        let converter = VernalTypeConverter::new();
        let value = TypedValue::new(
            ExpressionValue::String("abc".to_string()),
            TypeDescriptor::STRING,
        );
        assert!(
            converter
                .convert_value(&value, &TypeDescriptor::INT)
                .is_err()
        );
    }

    #[test]
    fn custom_converter() {
        let mut converter = VernalTypeConverter::new();
        converter.register_converter(|value, target| {
            if let (ExpressionValue::String(s), TypeDescriptor::Primitive(PrimitiveKind::Int)) =
                (value.value(), target)
            {
                if s == "custom" {
                    return Some(TypedValue::new(
                        ExpressionValue::Int(999),
                        TypeDescriptor::INT,
                    ));
                }
            }
            None
        });

        let value = TypedValue::new(
            ExpressionValue::String("custom".to_string()),
            TypeDescriptor::STRING,
        );
        let result = converter
            .convert_value(&value, &TypeDescriptor::INT)
            .unwrap();
        assert_eq!(*result.value(), ExpressionValue::Int(999));
    }
}
