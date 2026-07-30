//! AST 节点 trait（对标 Spring `org.springframework.expression.spel.SpelNode`）。

use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::TypedValue;

use super::super::expression_state::ExpressionState;

/// AST 节点 trait（对标 Spring `SpelNode`）。
///
/// Spring 的 `SpelNode.getValue(ExpressionState)` 是主要求值方法，
/// 允许通过 ExpressionState 操作上下文栈（push/pop active context）。
///
/// Rust 中提供两个方法：
/// - `get_value_state`：接收 `&mut ExpressionState`，支持上下文栈操作
/// - `get_value`：便捷方法，创建临时 ExpressionState 并委托
pub trait SpelNode: Send + Sync {
    /// 在给定上下文中求值（对标 Java `SpelNode.getValue(EvaluationContext)`）。
    ///
    /// 大多数节点实现此方法。需要操作上下文栈的节点
    /// （Selection/Projection/Indexer）应同时重写 `get_value_state`。
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException>;

    /// 在 ExpressionState 中求值（对标 Java `SpelNode.getValue(ExpressionState)`）。
    ///
    /// Selection/Projection/Indexer 等节点重写此方法以操作 active context 栈。
    /// 默认实现委托给 `get_value`。
    fn get_value_state(
        &self,
        state: &mut ExpressionState,
    ) -> Result<TypedValue, EvaluationException> {
        self.get_value(state.evaluation_context())
    }

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
