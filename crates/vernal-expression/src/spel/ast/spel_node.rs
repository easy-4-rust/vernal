//! AST 节点 trait（对标 Spring `org.springframework.expression.spel.SpelNode`）。

use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::TypedValue;

/// AST 节点 trait（对标 Spring `SpelNode`）。
pub trait SpelNode: Send + Sync {
    /// 在给定上下文中求值（对标 Java `SpelNode.getValue(EvaluationContext)`）。
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException>;

    /// 子节点数量（默认 0，对标 `getChildCount`）。
    fn child_count(&self) -> usize {
        0
    }

    /// 获取第 `i` 个子节点引用（默认 None，对标 `getChild(int)`）。
    fn get_child(&self, _i: usize) -> Option<&dyn SpelNode> {
        None
    }

    /// 节点是否为空安全（默认 false，对标 `isNullSafe()`）。
    fn is_null_safe(&self) -> bool {
        false
    }

    /// 起始位置（默认 0，对标 `getStartPosition()`）。
    fn start_position(&self) -> usize {
        0
    }

    /// 结束位置（默认 0，对标 `getEndPosition()`）。
    fn end_position(&self) -> usize {
        0
    }

    /// 节点运行时类型（对标 `getObjectClass()`）。
    fn class(&self) -> Option<std::any::TypeId> {
        None
    }

    /// 是否可写（默认 false）。
    fn is_writable(&self, _context: &dyn EvaluationContext) -> bool {
        false
    }

    /// 输出 AST 字符串表示（对标 `toStringAST()`）。
    fn to_string_ast(&self) -> String;

    /// 退出类型描述符。
    fn exit_descriptor(&self) -> Option<String> {
        None
    }
}

impl std::fmt::Debug for dyn SpelNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpelNode")
            .field("ast", &self.to_string_ast())
            .finish()
    }
}
