//! 方法执行器 trait。
//!
//! 对标 Spring 的 `MethodExecutor`。
use super::access_exception::AccessException;
use super::evaluation_context::EvaluationContext;
use super::typed_value::TypedValue;
pub trait MethodExecutor: Send + Sync {
    fn execute(&self, context: &dyn EvaluationContext, target: &TypedValue, arguments: &[TypedValue]) -> Result<TypedValue, AccessException>;
    fn box_clone(&self) -> Box<dyn MethodExecutor>;
}
