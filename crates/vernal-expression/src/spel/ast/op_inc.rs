//! 自增运算符。
//!
//! 对标 Spring 的 `OpInc`：`++`（前缀/后缀）

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 自增运算符节点。
///
/// 支持前缀（`++a`）和后缀（`a++`）两种形式。
pub struct OpInc {
    operand: Box<dyn SpelNode>,
    /// true 表示前缀，false 表示后缀
    prefix: bool,
}

impl OpInc {
    #[must_use]
    pub fn new(operand: Box<dyn SpelNode>, prefix: bool) -> Self {
        Self { operand, prefix }
    }
}

impl SpelNode for OpInc {
    fn get_value(
        &self,
        _context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        // 简化实现：仅返回值，不修改原值
        // 完整实现需要写回上下文
        let value = self.operand.get_value(_context)?;
        let new_value = match value.value() {
            ExpressionValue::Int(i) => ExpressionValue::Int(i + 1),
            ExpressionValue::Float(f) => ExpressionValue::Float(f + 1.0),
            _ => return Err(EvaluationException::new("", None, "自增运算只支持数值类型")),
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
            format!("++{}", self.operand.to_string_ast())
        } else {
            format!("{}++", self.operand.to_string_ast())
        }
    }
}
