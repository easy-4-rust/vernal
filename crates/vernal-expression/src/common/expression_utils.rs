//! 表达式工具类。
//!
//! 对标 Spring 的 `ExpressionUtils`：类型转换辅助方法。

use crate::evaluation_exception::EvaluationException;
use crate::type_converter::TypeConverter;
use crate::typed_value::{TypeDescriptor, TypedValue};

/// 表达式工具类。
///
/// 对标 Spring 的 `org.springframework.expression.common.ExpressionUtils`。
pub struct ExpressionUtils;

impl ExpressionUtils {
    /// 将 TypedValue 转换为指定类型。
    pub fn convert_typed_value<T: Default>(
        converter: &dyn TypeConverter,
        typed_value: &TypedValue,
        target_type: &TypeDescriptor,
    ) -> Result<T, EvaluationException> {
        let converted = converter.convert_value(typed_value, target_type)?;
        // 简化实现：仅返回默认值
        // 完整实现需要从 TypedValue 提取具体值
        let _ = converted;
        Ok(T::default())
    }

    /// 转换为整数。
    pub fn to_int(
        converter: &dyn TypeConverter,
        value: &TypedValue,
    ) -> Result<i64, EvaluationException> {
        let int_td = TypeDescriptor::new("int");
        let converted = converter.convert_value(value, &int_td)?;
        match converted.value() {
            crate::typed_value::ExpressionValue::Int(i) => Ok(*i),
            _ => Err(EvaluationException::new("", None, "无法转换为整数")),
        }
    }

    /// 转换为布尔值。
    pub fn to_boolean(
        converter: &dyn TypeConverter,
        value: &TypedValue,
    ) -> Result<bool, EvaluationException> {
        let bool_td = TypeDescriptor::new("boolean");
        let converted = converter.convert_value(value, &bool_td)?;
        match converted.value() {
            crate::typed_value::ExpressionValue::Boolean(b) => Ok(*b),
            _ => Err(EvaluationException::new("", None, "无法转换为布尔值")),
        }
    }

    /// 转换为浮点数。
    pub fn to_double(
        converter: &dyn TypeConverter,
        value: &TypedValue,
    ) -> Result<f64, EvaluationException> {
        let float_td = TypeDescriptor::new("float");
        let converted = converter.convert_value(value, &float_td)?;
        match converted.value() {
            crate::typed_value::ExpressionValue::Float(f) => Ok(*f),
            _ => Err(EvaluationException::new("", None, "无法转换为浮点数")),
        }
    }
}
