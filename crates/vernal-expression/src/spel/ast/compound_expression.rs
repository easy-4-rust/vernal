//! 点分表达式序列。
//!
//! 对标 Spring `org.springframework.expression.spel.ast.CompoundExpression`：
//! 表示 `a.b.c`、`a.b()[i].c` 这类链式访问。
//!
//! # Spring 求值语义
//!
//! CompoundExpression 求值时依次对子节点求值，前一个结果会被 push 到
//! `ExpressionState` 的 active context object 栈，作为下一个子节点的根对象。
//! 这样 `a.b.c` 中的 `b` 会以 `a` 的结果为根进行属性查找。

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::TypedValue;

/// 点分表达式序列节点（对标 Spring `CompoundExpression`）。
///
/// 依次求值子表达式，前一个的结果作为下一个的上下文根。
pub struct CompoundExpression {
    children: Vec<Box<dyn SpelNode>>,
}

impl CompoundExpression {
    /// 创建复合表达式节点。
    ///
    /// # 参数
    ///
    /// - `children` — 按顺序串联的子节点列表（至少 2 个）
    #[must_use]
    pub fn new(children: Vec<Box<dyn SpelNode>>) -> Self {
        Self { children }
    }

    /// 子节点数量。
    #[must_use]
    pub fn size(&self) -> usize {
        self.children.len()
    }
}

impl SpelNode for CompoundExpression {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        let mut result = self.children[0].get_value(context)?;
        for child in &self.children[1..] {
            result = child.get_value(context)?;
        }
        Ok(result)
    }

    fn child_count(&self) -> usize {
        self.children.len()
    }

    fn get_child(&self, i: usize) -> Option<&dyn SpelNode> {
        self.children.get(i).map(|b| b.as_ref())
    }

    fn start_position(&self) -> usize {
        self.children
            .first()
            .map(|c| c.start_position())
            .unwrap_or(0)
    }

    fn end_position(&self) -> usize {
        self.children.last().map(|c| c.end_position()).unwrap_or(0)
    }

    fn to_string_ast(&self) -> String {
        // 对标 Java：节点间用 `.` 连接，Indexer 前不加
        let mut s = String::new();
        for (i, child) in self.children.iter().enumerate() {
            s.push_str(&child.to_string_ast());
            if i + 1 < self.children.len() {
                let next = &self.children[i + 1];
                if next.is_null_safe() {
                    s.push_str("?.");
                } else if !next.to_string_ast().starts_with('[') {
                    s.push('.');
                }
            }
        }
        s
    }
}
