//! 逻辑与运算符。
//!
//! 对标 Spring 的 `OpAnd`：`&&`（短路求值）

use crate::typed_value::{TypedValue, ExpressionValue, TypeDescriptor};
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use super::spel_node::SpelNode;

/// 逻辑与运算符节点。

pub struct OpAnd { left: Box<dyn SpelNode>, right: Box<dyn SpelNode> }

impl OpAnd {
    #[must_use] pub fn new(left: Box<dyn SpelNode>, right: Box<dyn SpelNode>) -> Self { Self { left, right } }
}

impl SpelNode for OpAnd {
    fn get_value(&self, context: &dyn EvaluationContext) -> Result<TypedValue, EvaluationException> {
        let left = self.left.get_value(context)?;
        // 短路求值：左操作数为 false 时直接返回
        if let ExpressionValue::Boolean(false) = left.value() {
            return Ok(TypedValue::new(ExpressionValue::Boolean(false), TypeDescriptor::BOOLEAN));
        }
        let right = self.right.get_value(context)?;
        let result = matches!(right.value(), ExpressionValue::Boolean(true));
        Ok(TypedValue::new(ExpressionValue::Boolean(result), TypeDescriptor::BOOLEAN))
    }

    fn to_string_ast(&self) -> String {
        format!("({} && {})", self.left.to_string_ast(), self.right.to_string_ast())
    }
}
