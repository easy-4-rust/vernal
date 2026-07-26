//! AST 节点 trait。
//!
//! 对标 Spring 的 `SpelNode` 接口。

use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::TypedValue;

/// AST 节点 trait。
///
/// 对标 Spring 的 `org.springframework.expression.spel.SpelNode`。
pub trait SpelNode: Send + Sync {
    /// 在给定上下文中求值。
    fn get_value(&self, context: &dyn EvaluationContext)
    -> Result<TypedValue, EvaluationException>;

    /// 获取子节点数量。
    fn child_count(&self) -> usize {
        0
    }

    /// 获取表达式字符串表示。
    fn to_string_ast(&self) -> String;

    /// 节点是否可写。
    fn is_writable(&self) -> bool {
        false
    }
}
