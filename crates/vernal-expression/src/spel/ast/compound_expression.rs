//! 点分表达式序列。
//!
//! 对标 Spring 的 `CompoundExpression`：`a.b.c`

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::TypedValue;

/// 点分表达式序列节点。
///
/// 依次求值子表达式，前一个的结果作为下一个的上下文。
/// 对标 Spring 的 `org.springframework.expression.spel.ast.CompoundExpression`。
pub struct CompoundExpression {
    children: Vec<Box<dyn SpelNode>>,
}

impl CompoundExpression {
    #[must_use]
    pub fn new(children: Vec<Box<dyn SpelNode>>) -> Self {
        Self { children }
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

    fn to_string_ast(&self) -> String {
        self.children
            .iter()
            .map(|c| c.to_string_ast())
            .collect::<Vec<_>>()
            .join(".")
    }
}
