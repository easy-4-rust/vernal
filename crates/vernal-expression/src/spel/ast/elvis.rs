//! Elvis 运算符节点。
//!
//! 对标 Spring 的 `Elvis`：`A ?: B`

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypedValue};

/// Elvis 运算符节点。
///
/// 如果 A 非空则返回 A，否则返回 B。
pub struct Elvis {
    expression: Box<dyn SpelNode>,
    default: Box<dyn SpelNode>,
}

impl Elvis {
    #[must_use]
    pub fn new(expression: Box<dyn SpelNode>, default: Box<dyn SpelNode>) -> Self {
        Self {
            expression,
            default,
        }
    }
}

impl SpelNode for Elvis {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        let value = self.expression.get_value(context)?;
        match value.value() {
            ExpressionValue::Null => self.default.get_value(context),
            ExpressionValue::String(s) if s.is_empty() => self.default.get_value(context),
            _ => Ok(value),
        }
    }

    fn to_string_ast(&self) -> String {
        format!(
            "({} ?: {})",
            self.expression.to_string_ast(),
            self.default.to_string_ast()
        )
    }
}
