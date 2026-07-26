//! 不等于运算符。
//!
//! 对标 Spring 的 `OpNE`：`!=`

use crate::typed_value::{TypedValue, ExpressionValue, TypeDescriptor};
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use super::spel_node::SpelNode;
use super::operator::BinaryOperator;

/// 不等于运算符节点。

pub struct OpNe { left: Box<dyn SpelNode>, right: Box<dyn SpelNode> }

impl OpNe {
    #[must_use] pub fn new(left: Box<dyn SpelNode>, right: Box<dyn SpelNode>) -> Self { Self { left, right } }
}

impl SpelNode for OpNe {
    fn get_value(&self, context: &dyn EvaluationContext) -> Result<TypedValue, EvaluationException> {
        let left = self.left.get_value(context)?;
        let right = self.right.get_value(context)?;
        self.operate(&left, &right)
    }
    fn to_string_ast(&self) -> String { format!("({} != {})", self.left.to_string_ast(), self.right.to_string_ast()) }
}

impl BinaryOperator for OpNe {
    fn left(&self) -> &dyn SpelNode { &*self.left }
    fn right(&self) -> &dyn SpelNode { &*self.right }
    fn operator_name(&self) -> &str { "!=" }

    fn operate(&self, left: &TypedValue, right: &TypedValue) -> Result<TypedValue, EvaluationException> {
        let result = match (left.value(), right.value()) {
            (ExpressionValue::Int(l), ExpressionValue::Int(r)) => l != r,
            (ExpressionValue::Float(l), ExpressionValue::Float(r)) => (l - r).abs() >= f64::EPSILON,
            (ExpressionValue::String(l), ExpressionValue::String(r)) => l != r,
            (ExpressionValue::Boolean(l), ExpressionValue::Boolean(r)) => l != r,
            (ExpressionValue::Null, ExpressionValue::Null) => false,
            _ => true,
        };
        Ok(TypedValue::new(ExpressionValue::Boolean(result), TypeDescriptor::BOOLEAN))
    }
}
