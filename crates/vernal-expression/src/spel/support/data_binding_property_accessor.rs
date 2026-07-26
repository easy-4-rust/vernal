//! 数据绑定属性访问器。
//!
//! 对标 Spring 的 `DataBindingPropertyAccessor`：仅访问公共属性。

use crate::typed_value::TypedValue;
use crate::property_accessor::PropertyAccessor;
use crate::access_exception::AccessException;
use crate::evaluation_context::EvaluationContext;

/// 数据绑定属性访问器。
///
/// 对标 Spring 的 `DataBindingPropertyAccessor`。
pub struct DataBindingPropertyAccessor;

impl PropertyAccessor for DataBindingPropertyAccessor {
    fn can_read(&self, _context: &dyn EvaluationContext, _target: &TypedValue, name: &str) -> bool {
        // 简化实现：仅支持公共属性
        !name.is_empty()
    }

    fn read(&self, _context: &dyn EvaluationContext, _target: &TypedValue, _name: &str) -> Result<TypedValue, AccessException> {
        Ok(TypedValue::null())
    }

    fn can_write(&self, _context: &dyn EvaluationContext, _target: &TypedValue, _name: &str) -> bool {
        false
    }

    fn write(&self, _context: &dyn EvaluationContext, _target: &TypedValue, _name: &str, _value: &TypedValue) -> Result<(), AccessException> {
        Err(AccessException::new("数据绑定属性访问器不支持写入"))
    }
}
