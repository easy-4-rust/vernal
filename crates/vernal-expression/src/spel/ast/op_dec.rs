//! 自减运算符。
//!
//! 对标 Spring 的 `OpDec`：`--`（前缀/后缀）

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 自减运算符节点。
///
/// 支持前缀（`--a`）和后缀（`a--`）两种形式。
pub struct OpDec {
    operand: Box<dyn SpelNode>,
    prefix: bool,
}

impl OpDec {
    #[must_use]
    pub fn new(operand: Box<dyn SpelNode>, prefix: bool) -> Self {
        Self { operand, prefix }
    }
}

impl SpelNode for OpDec {
    fn get_value(
        &self,
        _context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        let value = self.operand.get_value(_context)?;
        let new_value = match value.value() {
            ExpressionValue::Int(i) => ExpressionValue::Int(i - 1),
            ExpressionValue::Float(f) => ExpressionValue::Float(f - 1.0),
            _ => return Err(EvaluationException::new("", None, "自减运算只支持数值类型")),
        };
        let result = if self.prefix {
            new_value.clone()
        } else {
            value.value().clone()
        };
        Ok(TypedValue::new(result, value.type_descriptor().clone()))
    }

    fn to_string_ast(&self) -> String {
        if self.prefix {
            format!("--{}", self.operand.to_string_ast())
        } else {
            format!("{}--", self.operand.to_string_ast())
        }
    }
}
