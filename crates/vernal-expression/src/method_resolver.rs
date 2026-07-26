//! 方法解析器 trait。
//!
//! 对标 Spring 的 `MethodResolver`。

use super::access_exception::AccessException;
use super::evaluation_context::EvaluationContext;
use super::method_executor::MethodExecutor;
use super::typed_value::{TypeDescriptor, TypedValue};

/// 方法解析器 trait。
///
/// 定位方法并返回 [`MethodExecutor`]。
/// 对标 Spring 的 `org.springframework.expression.MethodResolver`。
pub trait MethodResolver: Send + Sync {
    /// 解析方法。
    fn resolve(
        &self,
        context: &dyn EvaluationContext,
        target: &TypedValue,
        name: &str,
        argument_types: &[TypeDescriptor],
    ) -> Result<Option<Box<dyn MethodExecutor>>, AccessException>;
}
