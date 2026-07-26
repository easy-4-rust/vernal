//! 赋值表达式节点。
//!
//! 对标 Spring 的 `Assign`：`lhs = rhs`

use crate::typed_value::TypedValue;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use super::spel_node::SpelNode;

/// 赋值表达式节点。
pub struct Assign {
    left: Box<dyn SpelNode>,
    right: Box<dyn SpelNode>,
}

impl Assign {
    #[must_use]
    pub fn new(left: Box<dyn SpelNode>, right: Box<dyn SpelNode>) -> Self {
        Self { left, right }
    }
}

impl SpelNode for Assign {
    fn get_value(&self, context: &dyn EvaluationContext) -> Result<TypedValue, EvaluationException> {
        if !context.is_assignment_enabled() {
            return Err(EvaluationException::new("", None, "赋值操作未启用"));
        }
        let value = self.right.get_value(context)?;
        // 简化实现：直接返回右值
        Ok(value)
    }

    fn is_writable(&self) -> bool {
        true
    }

    fn to_string_ast(&self) -> String {
        format!("({} = {})", self.left.to_string_ast(), self.right.to_string_ast())
    }
}
