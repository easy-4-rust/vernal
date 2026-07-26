//! 逻辑非运算符。
//!
//! 对标 Spring 的 `OperatorNot`：`!`

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 逻辑非运算符节点。

pub struct OperatorNot {
    operand: Box<dyn SpelNode>,
}

impl OperatorNot {
    #[must_use]
    pub fn new(operand: Box<dyn SpelNode>) -> Self {
        Self { operand }
    }
}

impl SpelNode for OperatorNot {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        let value = self.operand.get_value(context)?;
        let result = match value.value() {
            ExpressionValue::Boolean(b) => !b,
            _ => {
                return Err(EvaluationException::new(
                    "",
                    None,
                    "逻辑非运算不支持的操作数类型",
                ));
            }
        };
        Ok(TypedValue::new(
            ExpressionValue::Boolean(result),
            TypeDescriptor::BOOLEAN,
        ))
    }

    fn to_string_ast(&self) -> String {
        format!("!({})", self.operand.to_string_ast())
    }
}
