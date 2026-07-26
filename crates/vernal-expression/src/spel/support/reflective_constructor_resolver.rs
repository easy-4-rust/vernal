//! 反射构造器解析器。
//!
//! 对标 Spring 的 `ReflectiveConstructorResolver`。

use crate::access_exception::AccessException;
use crate::constructor_executor::ConstructorExecutor;
use crate::constructor_resolver::ConstructorResolver;
use crate::evaluation_context::EvaluationContext;
use crate::typed_value::{TypeDescriptor, TypedValue};

/// 反射构造器解析器。
///
/// 对标 Spring 的 `ReflectiveConstructorResolver`。
pub struct ReflectiveConstructorResolver;

impl ConstructorResolver for ReflectiveConstructorResolver {
    fn resolve(
        &self,
        _context: &dyn EvaluationContext,
        _type_name: &str,
        _argument_types: &[TypeDescriptor],
    ) -> Result<Option<Box<dyn ConstructorExecutor>>, AccessException> {
        // 简化实现：返回 None（未找到）
        // 完整实现需要通过反射查找构造器
        Ok(None)
    }
}

/// 反射构造器执行器。
///
/// 对标 Spring 的 `ReflectiveConstructorExecutor`。
pub struct ReflectiveConstructorExecutor;

impl ConstructorExecutor for ReflectiveConstructorExecutor {
    fn execute(
        &self,
        _context: &dyn EvaluationContext,
        _arguments: &[TypedValue],
    ) -> Result<TypedValue, AccessException> {
        Err(AccessException::new("反射构造器执行未实现"))
    }
}
