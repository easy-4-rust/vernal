//! 反射属性访问器。
//!
//! 对标 Spring 的 `ReflectivePropertyAccessor`：通过反射访问属性。
//! Rust 中通过 trait 方法模拟反射。

use std::any::Any;
use crate::typed_value::{TypedValue, ExpressionValue, TypeDescriptor};
use crate::property_accessor::PropertyAccessor;
use crate::access_exception::AccessException;
use crate::evaluation_context::EvaluationContext;

/// 反射属性访问器。
///
/// 对标 Spring 的 `ReflectivePropertyAccessor`。
/// Rust 中通过 trait 方法模拟反射行为。
pub struct ReflectivePropertyAccessor;

impl PropertyAccessor for ReflectivePropertyAccessor {
    fn can_read(&self, _context: &dyn EvaluationContext, target: &TypedValue, name: &str) -> bool {
        // 通过 trait 方法检查目标是否有对应属性
        let _ = (target, name);
        // 简化实现：返回 true
        true
    }

    fn read(
        &self,
        _context: &dyn EvaluationContext,
        target: &TypedValue,
        name: &str,
    ) -> Result<TypedValue, AccessException> {
        let _ = (target, name);
        Ok(TypedValue::null())
    }

    fn can_write(&self, _context: &dyn EvaluationContext, _target: &TypedValue, _name: &str) -> bool {
        false
    }

    fn write(
        &self,
        _context: &dyn EvaluationContext,
        _target: &TypedValue,
        _name: &str,
        _value: &TypedValue,
    ) -> Result<(), AccessException> {
        Err(AccessException::new("反射属性访问器不支持写入"))
    }
}
