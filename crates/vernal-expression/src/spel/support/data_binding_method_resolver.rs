//! 数据绑定方法解析器。
//!
//! 对标 Spring 的 `DataBindingMethodResolver`：仅解析目标对象上的方法。

use crate::typed_value::{TypedValue, TypeDescriptor};
use crate::method_resolver::MethodResolver;
use crate::access_exception::AccessException;
use crate::evaluation_context::EvaluationContext;
use crate::method_executor::MethodExecutor;

/// 数据绑定方法解析器。
///
/// 对标 Spring 的 `DataBindingMethodResolver`。
pub struct DataBindingMethodResolver;

impl MethodResolver for DataBindingMethodResolver {
    fn resolve(
        &self,
        _context: &dyn EvaluationContext,
        _target: &TypedValue,
        _name: &str,
        _argument_types: &[TypeDescriptor],
    ) -> Result<Option<Box<dyn MethodExecutor>>, AccessException> {
        // 简化实现：返回 None
        // 完整实现需要通过反射查找目标对象的方法
        Ok(None)
    }
}
