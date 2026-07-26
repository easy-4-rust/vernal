//! 方法执行器 trait。
//!
//! 对标 Spring 的 `MethodExecutor`。

use super::access_exception::AccessException;
use super::evaluation_context::EvaluationContext;
use super::typed_value::TypedValue;

/// 方法执行器 trait。
///
/// 执行已解析的方法。可被缓存以提高性能。
/// 对标 Spring 的 `org.springframework.expression.MethodExecutor`。
pub trait MethodExecutor: Send + Sync {
    /// 执行方法。
    fn execute(
        &self,
        context: &dyn EvaluationContext,
        target: &TypedValue,
        arguments: &[TypedValue],
    ) -> Result<TypedValue, AccessException>;
}
