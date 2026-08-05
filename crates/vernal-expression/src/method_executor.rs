//! 方法执行器 trait。
//!
//! 对标 Spring 的 `MethodExecutor`。
use super::access_exception::AccessException;
use super::evaluation_context::EvaluationContext;
use super::typed_value::TypedValue;

/// 方法执行器（对标 `org.springframework.expression.MethodExecutor`）。
///
/// 负责在求值过程中执行已解析的方法。
pub trait MethodExecutor: Send + Sync {
    /// 执行方法并返回结果值。
    ///
    /// # 参数
    ///
    /// - `context` — 求值上下文
    /// - `target` — 目标对象
    /// - `arguments` — 方法参数列表
    fn execute(
        &self,
        context: &dyn EvaluationContext,
        target: &TypedValue,
        arguments: &[TypedValue],
    ) -> Result<TypedValue, AccessException>;
}
