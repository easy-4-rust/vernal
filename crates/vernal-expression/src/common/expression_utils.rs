//! 表达式工具类。
//!
//! 对标 Spring 的 `ExpressionUtils`：类型转换辅助方法。

use crate::evaluation_exception::EvaluationException;
use crate::type_converter::TypeConverter;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 表达式工具类。
///
/// 对标 Spring 的 `org.springframework.expression.common.ExpressionUtils`。
pub struct ExpressionUtils;

impl ExpressionUtils {
    /// 将 TypedValue 转换为指定类型（对标 Spring `ExpressionUtils.convertTypedValue`）。
    ///
    /// 返回转换后的 `TypedValue`，调用者可从 `value()` 中提取具体值。
    pub fn convert_typed_value(
        converter: &dyn TypeConverter,
        typed_value: &TypedValue,
        target_type: &TypeDescriptor,
    ) -> Result<TypedValue, EvaluationException> {
        converter.convert_value(typed_value, target_type)
    }

    /// 转换为整数（对标 Spring `ExpressionUtils.toInt`）。
    pub fn to_int(
        converter: &dyn TypeConverter,
        value: &TypedValue,
    ) -> Result<i64, EvaluationException> {
        let int_td = TypeDescriptor::INT;
        let converted = converter.convert_value(value, &int_td)?;
        match converted.value() {
            ExpressionValue::Int(i) => Ok(*i),
            _ => Err(EvaluationException::new("", None, "无法转换为整数")),
        }
    }

    /// 转换为布尔值（对标 Spring `ExpressionUtils.toBoolean`）。
    pub fn to_boolean(
        converter: &dyn TypeConverter,
        value: &TypedValue,
    ) -> Result<bool, EvaluationException> {
        let bool_td = TypeDescriptor::BOOLEAN;
        let converted = converter.convert_value(value, &bool_td)?;
        match converted.value() {
            ExpressionValue::Boolean(b) => Ok(*b),
            _ => Err(EvaluationException::new("", None, "无法转换为布尔值")),
        }
    }

    /// 转换为浮点数（对标 Spring `ExpressionUtils.toDouble`）。
    pub fn to_double(
        converter: &dyn TypeConverter,
        value: &TypedValue,
    ) -> Result<f64, EvaluationException> {
        let float_td = TypeDescriptor::FLOAT;
        let converted = converter.convert_value(value, &float_td)?;
        match converted.value() {
            ExpressionValue::Float(f) => Ok(*f),
            ExpressionValue::Double(d) => Ok(*d),
            _ => Err(EvaluationException::new("", None, "无法转换为浮点数")),
        }
    }

    /// 转换为字符串（对标 Spring `ExpressionUtils.toTypedValue` with String）。
    pub fn to_string_value(
        converter: &dyn TypeConverter,
        value: &TypedValue,
    ) -> Result<String, EvaluationException> {
        let string_td = TypeDescriptor::STRING;
        let converted = converter.convert_value(value, &string_td)?;
        match converted.value() {
            ExpressionValue::String(s) => Ok(s.clone()),
            _ => Err(EvaluationException::new("", None, "无法转换为字符串")),
        }
    }
}
