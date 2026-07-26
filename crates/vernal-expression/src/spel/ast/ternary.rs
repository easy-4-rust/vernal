//! 三元表达式节点。
//!
//! 对标 Spring 的 `Ternary`：`condition ? trueValue : falseValue`

use crate::typed_value::{TypedValue, ExpressionValue};
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use super::spel_node::SpelNode;

/// 三元表达式节点。
pub struct Ternary {
    condition: Box<dyn SpelNode>,
    true_value: Box<dyn SpelNode>,
    false_value: Box<dyn SpelNode>,
}

impl Ternary {
    #[must_use]
    pub fn new(condition: Box<dyn SpelNode>, true_value: Box<dyn SpelNode>, false_value: Box<dyn SpelNode>) -> Self {
        Self { condition, true_value, false_value }
    }
}

impl SpelNode for Ternary {
    fn get_value(&self, context: &dyn EvaluationContext) -> Result<TypedValue, EvaluationException> {
        let cond = self.condition.get_value(context)?;
        match cond.value() {
            ExpressionValue::Boolean(true) => self.true_value.get_value(context),
            ExpressionValue::Boolean(false) => self.false_value.get_value(context),
            _ => Err(EvaluationException::new("", None, "三元表达式条件必须是布尔值")),
        }
    }

    fn to_string_ast(&self) -> String {
        format!("({} ? {} : {})", self.condition.to_string_ast(), self.true_value.to_string_ast(), self.false_value.to_string_ast())
    }
}
