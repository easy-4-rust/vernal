//! Vernal IoC 容器属性访问器。
//!
//! 从 Vernal ApplicationContext 注入属性，对标 Spring 的 BeanFactory 集成。

use crate::access_exception::AccessException;
use crate::evaluation_context::EvaluationContext;
use crate::property_accessor::PropertyAccessor;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// Vernal 属性访问器。
///
/// 从 Vernal IoC 容器解析属性。
/// 这是 Rust 侧新增功能，对标 Spring 的 BeanFactory 集成。
pub struct VernalPropertyAccessor;

impl PropertyAccessor for VernalPropertyAccessor {
    fn can_read(
        &self,
        _context: &dyn EvaluationContext,
        _target: &TypedValue,
        _name: &str,
    ) -> bool {
        // 总是尝试读取（由 Container 决定是否成功）
        true
    }

    fn read(
        &self,
        _context: &dyn EvaluationContext,
        _target: &TypedValue,
        name: &str,
    ) -> Result<TypedValue, AccessException> {
        // 简化实现：返回属性名作为字符串
        // 完整实现需要从 Vernal Container 解析
        Ok(TypedValue::new(
            ExpressionValue::String(name.to_string()),
            TypeDescriptor::STRING,
        ))
    }

    fn can_write(
        &self,
        _context: &dyn EvaluationContext,
        _target: &TypedValue,
        _name: &str,
    ) -> bool {
        false
    }

    fn write(
        &self,
        _context: &dyn EvaluationContext,
        _target: &TypedValue,
        _name: &str,
        _value: &TypedValue,
    ) -> Result<(), AccessException> {
        Err(AccessException::new("VernalPropertyAccessor 不支持写入"))
    }
}
