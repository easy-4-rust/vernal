//! 标准类型转换器。
//!
//! 对标 Spring 的 `StandardTypeConverter`。

use crate::evaluation_exception::EvaluationException;
use crate::type_converter::TypeConverter;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 标准类型转换器。
///
/// 对标 Spring 的 `org.springframework.expression.spel.support.StandardTypeConverter`。
pub struct StandardTypeConverter;

impl TypeConverter for StandardTypeConverter {
    fn can_convert(&self, _source_type: &TypeDescriptor, _target_type: &TypeDescriptor) -> bool {
        true
    }

    fn convert_value(
        &self,
        value: &TypedValue,
        target_type: &TypeDescriptor,
    ) -> Result<TypedValue, EvaluationException> {
        // 简化实现：基本类型转换
        match (value.value(), target_type) {
            (ExpressionValue::Int(i), td) if *td == TypeDescriptor::STRING => Ok(TypedValue::new(
                ExpressionValue::String(i.to_string()),
                TypeDescriptor::STRING,
            )),
            (ExpressionValue::Float(f), td) if *td == TypeDescriptor::STRING => {
                Ok(TypedValue::new(
                    ExpressionValue::String(f.to_string()),
                    TypeDescriptor::STRING,
                ))
            }
            (ExpressionValue::Boolean(b), td) if *td == TypeDescriptor::STRING => {
                Ok(TypedValue::new(
                    ExpressionValue::String(b.to_string()),
                    TypeDescriptor::STRING,
                ))
            }
            (ExpressionValue::String(s), td) if *td == TypeDescriptor::INT => s
                .parse::<i64>()
                .map(|i| TypedValue::new(ExpressionValue::Int(i), TypeDescriptor::INT))
                .map_err(|_| {
                    EvaluationException::new("", None, format!("无法将 '{}' 转换为整数", s))
                }),
            (ExpressionValue::String(s), td) if *td == TypeDescriptor::FLOAT => s
                .parse::<f64>()
                .map(|f| TypedValue::new(ExpressionValue::Float(f), TypeDescriptor::FLOAT))
                .map_err(|_| {
                    EvaluationException::new("", None, format!("无法将 '{}' 转换为浮点数", s))
                }),
            _ => Ok(value.clone()),
        }
    }
}
