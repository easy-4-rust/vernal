//! 反射方法解析器。
//!
//! 对标 Spring 的 `ReflectiveMethodResolver`。

use std::any::TypeId;
use crate::typed_value::{TypedValue, TypeDescriptor};
use crate::method_resolver::MethodResolver;
use crate::access_exception::AccessException;
use crate::evaluation_context::EvaluationContext;
use crate::method_executor::MethodExecutor;

/// 反射方法解析器。
///
/// 对标 Spring 的 `ReflectiveMethodResolver`。
pub struct ReflectiveMethodResolver;

impl MethodResolver for ReflectiveMethodResolver {
    fn resolve(
        &self,
        _context: &dyn EvaluationContext,
        _target: &TypedValue,
        _name: &str,
        _argument_types: &[TypeDescriptor],
    ) -> Result<Option<Box<dyn MethodExecutor>>, AccessException> {
        // 简化实现：返回 None（未找到）
        // 完整实现需要通过反射查找方法
        Ok(None)
    }
}

/// 反射方法执行器。
///
/// 对标 Spring 的 `ReflectiveMethodExecutor`。
pub struct ReflectiveMethodExecutor;

impl MethodExecutor for ReflectiveMethodExecutor {
    fn execute(
        &self,
        _context: &dyn EvaluationContext,
        _target: &TypedValue,
        _arguments: &[TypedValue],
    ) -> Result<TypedValue, AccessException> {
        Err(AccessException::new("反射方法执行未实现"))
    }
}

/// 反射方法解析器占位。
pub struct ReflectiveMethodExecutorPlaceholder {
    type_id: TypeId,
}

impl ReflectiveMethodExecutorPlaceholder {
    /// 创建占位符。
    #[must_use]
    pub fn new(type_id: TypeId) -> Self {
        Self { type_id }
    }
}
