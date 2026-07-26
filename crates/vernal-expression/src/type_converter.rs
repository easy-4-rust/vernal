//! 类型转换器 trait。
//!
//! 对标 Spring 的 `TypeConverter`。

use super::evaluation_exception::EvaluationException;
use super::typed_value::{TypeDescriptor, TypedValue};

/// 类型转换器 trait。
///
/// 在表达式求值期间转换值的类型。
/// 对标 Spring 的 `org.springframework.expression.TypeConverter`。
pub trait TypeConverter: Send + Sync {
    /// 是否可以执行转换。
    fn can_convert(&self, source_type: &TypeDescriptor, target_type: &TypeDescriptor) -> bool;

    /// 执行类型转换。
    fn convert_value(
        &self,
        value: &TypedValue,
        target_type: &TypeDescriptor,
    ) -> Result<TypedValue, EvaluationException>;
}
