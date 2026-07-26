//! between 运算符。
//!
//! 对标 Spring 的 `OperatorBetween`：`value between [low, high]`

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// between 运算符节点。
///
/// 检查值是否在两个边界之间（含边界）。
pub struct OperatorBetween {
    value: Box<dyn SpelNode>,
    low: Box<dyn SpelNode>,
    high: Box<dyn SpelNode>,
}

impl OperatorBetween {
    #[must_use]
    pub fn new(value: Box<dyn SpelNode>, low: Box<dyn SpelNode>, high: Box<dyn SpelNode>) -> Self {
        Self { value, low, high }
    }
}

impl SpelNode for OperatorBetween {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        let value = self.value.get_value(context)?;
        let low = self.low.get_value(context)?;
        let high = self.high.get_value(context)?;
        let result = match (value.value(), low.value(), high.value()) {
            (ExpressionValue::Int(v), ExpressionValue::Int(l), ExpressionValue::Int(h)) => {
                v >= l && v <= h
            }
            (ExpressionValue::Float(v), ExpressionValue::Float(l), ExpressionValue::Float(h)) => {
                v >= l && v <= h
            }
            (
                ExpressionValue::String(v),
                ExpressionValue::String(l),
                ExpressionValue::String(h),
            ) => v.as_str() >= l.as_str() && v.as_str() <= h.as_str(),
            _ => {
                return Err(EvaluationException::new(
                    "",
                    None,
                    "between 运算要求左右操作数类型一致",
                ));
            }
        };
        Ok(TypedValue::new(
            ExpressionValue::Boolean(result),
            TypeDescriptor::BOOLEAN,
        ))
    }

    fn to_string_ast(&self) -> String {
        format!(
            "({} between {} and {})",
            self.value.to_string_ast(),
            self.low.to_string_ast(),
            self.high.to_string_ast()
        )
    }
}
