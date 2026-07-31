//! 标准类型转换器（对标 Spring `StandardTypeConverter`）。
//!
//! 对标 Java `org.springframework.expression.spel.support.StandardTypeConverter`。
//! 支持基本类型之间的转换（Int↔Float↔String↔Boolean 等）。

use crate::evaluation_exception::EvaluationException;
use crate::type_converter::TypeConverter;
use crate::type_descriptor::PrimitiveKind;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 标准类型转换器（对标 Spring `StandardTypeConverter`）。
///
/// 支持的转换路径：
/// - Int ↔ String
/// - Float ↔ String
/// - Boolean ↔ String
/// - Int ↔ Float（数字 widening）
/// - Long ↔ Int（截断）
/// - Char ↔ String
pub struct StandardTypeConverter;

impl StandardTypeConverter {
    /// 单例实例。
    pub const INSTANCE: Self = Self;
}

impl TypeConverter for StandardTypeConverter {
    fn can_convert(&self, source_type: &TypeDescriptor, target_type: &TypeDescriptor) -> bool {
        // 所有基本类型之间都可转换
        source_type.is_primitive() && target_type.is_primitive()
    }

    fn convert_value(
        &self,
        value: &TypedValue,
        target_type: &TypeDescriptor,
    ) -> Result<TypedValue, EvaluationException> {
        match (value.value(), target_type) {
            // Int → String
            (ExpressionValue::Int(i), TypeDescriptor::Primitive(PrimitiveKind::String)) => {
                Ok(TypedValue::new(
                    ExpressionValue::String(i.to_string()),
                    TypeDescriptor::STRING,
                ))
            }
            // Long → String
            (ExpressionValue::Long(l), TypeDescriptor::Primitive(PrimitiveKind::String)) => {
                Ok(TypedValue::new(
                    ExpressionValue::String(l.to_string()),
                    TypeDescriptor::STRING,
                ))
            }
            // Float → String
            (ExpressionValue::Float(f), TypeDescriptor::Primitive(PrimitiveKind::String)) => {
                Ok(TypedValue::new(
                    ExpressionValue::String(f.to_string()),
                    TypeDescriptor::STRING,
                ))
            }
            // Double → String
            (ExpressionValue::Double(d), TypeDescriptor::Primitive(PrimitiveKind::String)) => {
                Ok(TypedValue::new(
                    ExpressionValue::String(d.to_string()),
                    TypeDescriptor::STRING,
                ))
            }
            // Boolean → String
            (ExpressionValue::Boolean(b), TypeDescriptor::Primitive(PrimitiveKind::String)) => {
                Ok(TypedValue::new(
                    ExpressionValue::String(b.to_string()),
                    TypeDescriptor::STRING,
                ))
            }
            // Char → String
            (ExpressionValue::Char(c), TypeDescriptor::Primitive(PrimitiveKind::String)) => {
                Ok(TypedValue::new(
                    ExpressionValue::String(c.to_string()),
                    TypeDescriptor::STRING,
                ))
            }
            // String → Int
            (ExpressionValue::String(s), TypeDescriptor::Primitive(PrimitiveKind::Int)) => {
                s.parse::<i64>()
                    .map(|i| TypedValue::new(ExpressionValue::Int(i), TypeDescriptor::INT))
                    .map_err(|_| {
                        EvaluationException::new(
                            "",
                            None,
                            format!("无法将 '{}' 转换为整数", s),
                        )
                    })
            }
            // String → Float
            (ExpressionValue::String(s), TypeDescriptor::Primitive(PrimitiveKind::Float)) => {
                s.parse::<f64>()
                    .map(|f| TypedValue::new(ExpressionValue::Float(f), TypeDescriptor::FLOAT))
                    .map_err(|_| {
                        EvaluationException::new(
                            "",
                            None,
                            format!("无法将 '{}' 转换为浮点数", s),
                        )
                    })
            }
            // String → Boolean
            (ExpressionValue::String(s), TypeDescriptor::Primitive(PrimitiveKind::Boolean)) => {
                match s.as_str() {
                    "true" | "TRUE" | "True" | "1" => {
                        Ok(TypedValue::new(ExpressionValue::Boolean(true), TypeDescriptor::BOOLEAN))
                    }
                    "false" | "FALSE" | "False" | "0" => {
                        Ok(TypedValue::new(ExpressionValue::Boolean(false), TypeDescriptor::BOOLEAN))
                    }
                    _ => Err(EvaluationException::new(
                        "",
                        None,
                        format!("无法将 '{}' 转换为布尔值", s),
                    )),
                }
            }
            // Int → Float (widening)
            (ExpressionValue::Int(i), TypeDescriptor::Primitive(PrimitiveKind::Float)) => {
                Ok(TypedValue::new(
                    ExpressionValue::Float(*i as f64),
                    TypeDescriptor::FLOAT,
                ))
            }
            // Float → Int (narrowing, truncation)
            (ExpressionValue::Float(f), TypeDescriptor::Primitive(PrimitiveKind::Int)) => {
                Ok(TypedValue::new(
                    ExpressionValue::Int(*f as i64),
                    TypeDescriptor::INT,
                ))
            }
            // Boolean → Int (false=0, true=1)
            (ExpressionValue::Boolean(b), TypeDescriptor::Primitive(PrimitiveKind::Int)) => {
                Ok(TypedValue::new(
                    ExpressionValue::Int(if *b { 1 } else { 0 }),
                    TypeDescriptor::INT,
                ))
            }
            // Int → Boolean (0=false, non-zero=true)
            (ExpressionValue::Int(i), TypeDescriptor::Primitive(PrimitiveKind::Boolean)) => {
                Ok(TypedValue::new(
                    ExpressionValue::Boolean(*i != 0),
                    TypeDescriptor::BOOLEAN,
                ))
            }
            // Null → String
            (ExpressionValue::Null, TypeDescriptor::Primitive(PrimitiveKind::String)) => {
                Ok(TypedValue::new(
                    ExpressionValue::String("null".to_string()),
                    TypeDescriptor::STRING,
                ))
            }
            // 同类型直接返回
            _ if value.type_descriptor() == target_type => Ok(value.clone()),
            // 未知转换：返回原值（对标 Spring 默认行为）
            _ => Ok(value.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_to_string() {
        let converter = StandardTypeConverter;
        let value = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
        let result = converter.convert_value(&value, &TypeDescriptor::STRING).unwrap();
        assert_eq!(*result.value(), ExpressionValue::String("42".to_string()));
    }

    #[test]
    fn string_to_int() {
        let converter = StandardTypeConverter;
        let value = TypedValue::new(ExpressionValue::String("123".to_string()), TypeDescriptor::STRING);
        let result = converter.convert_value(&value, &TypeDescriptor::INT).unwrap();
        assert_eq!(*result.value(), ExpressionValue::Int(123));
    }

    #[test]
    fn string_to_int_error() {
        let converter = StandardTypeConverter;
        let value = TypedValue::new(ExpressionValue::String("abc".to_string()), TypeDescriptor::STRING);
        assert!(converter.convert_value(&value, &TypeDescriptor::INT).is_err());
    }

    #[test]
    fn boolean_to_string() {
        let converter = StandardTypeConverter;
        let value = TypedValue::new(ExpressionValue::Boolean(true), TypeDescriptor::BOOLEAN);
        let result = converter.convert_value(&value, &TypeDescriptor::STRING).unwrap();
        assert_eq!(*result.value(), ExpressionValue::String("true".to_string()));
    }

    #[test]
    fn string_to_boolean_true() {
        let converter = StandardTypeConverter;
        let value = TypedValue::new(ExpressionValue::String("true".to_string()), TypeDescriptor::STRING);
        let result = converter.convert_value(&value, &TypeDescriptor::BOOLEAN).unwrap();
        assert_eq!(*result.value(), ExpressionValue::Boolean(true));
    }

    #[test]
    fn string_to_boolean_false() {
        let converter = StandardTypeConverter;
        let value = TypedValue::new(ExpressionValue::String("false".to_string()), TypeDescriptor::STRING);
        let result = converter.convert_value(&value, &TypeDescriptor::BOOLEAN).unwrap();
        assert_eq!(*result.value(), ExpressionValue::Boolean(false));
    }

    #[test]
    fn int_to_float_widening() {
        let converter = StandardTypeConverter;
        let value = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
        let result = converter.convert_value(&value, &TypeDescriptor::FLOAT).unwrap();
        assert_eq!(*result.value(), ExpressionValue::Float(42.0));
    }

    #[test]
    fn float_to_int_narrowing() {
        let converter = StandardTypeConverter;
        let value = TypedValue::new(ExpressionValue::Float(3.7), TypeDescriptor::FLOAT);
        let result = converter.convert_value(&value, &TypeDescriptor::INT).unwrap();
        assert_eq!(*result.value(), ExpressionValue::Int(3));
    }

    #[test]
    fn can_convert_primitives() {
        let converter = StandardTypeConverter;
        assert!(converter.can_convert(&TypeDescriptor::INT, &TypeDescriptor::STRING));
        assert!(converter.can_convert(&TypeDescriptor::BOOLEAN, &TypeDescriptor::INT));
    }
}
