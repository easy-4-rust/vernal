//! 构造器执行器 trait。
//!
//! 对标 Spring 的 `ConstructorExecutor`。

use super::access_exception::AccessException;
use super::evaluation_context::EvaluationContext;
use super::typed_value::TypedValue;

/// 构造器执行器 trait。
///
/// 执行已解析的构造器。
/// 对标 Spring 的 `org.springframework.expression.ConstructorExecutor`。
pub trait ConstructorExecutor: Send + Sync {
    /// 执行构造器。
    fn execute(
        &self,
        context: &dyn EvaluationContext,
        arguments: &[TypedValue],
    ) -> Result<TypedValue, AccessException>;
}
