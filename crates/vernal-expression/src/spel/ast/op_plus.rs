//! 加法运算符。
//!
//! 对标 Spring 的 `OpPlus`：`+`（加法、字符串连接、一元正号）

use crate::typed_value::{TypedValue, ExpressionValue, TypeDescriptor};
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use super::spel_node::SpelNode;
use super::operator::BinaryOperator;

/// 加法运算符节点。
///
/// 支持：
/// - 数值加法：`1 + 2` → `3`
/// - 字符串连接：`'hello' + ' world'` → `'hello world'`
/// - 一元正号：`+42` → `42`

pub struct OpPlus {
    left: Box<dyn SpelNode>,
    right: Box<dyn SpelNode>,
}

impl OpPlus {
    #[must_use]
    pub fn new(left: Box<dyn SpelNode>, right: Box<dyn SpelNode>) -> Self {
        Self { left, right }
    }
}

impl SpelNode for OpPlus {
    fn get_value(&self, context: &dyn EvaluationContext) -> Result<TypedValue, EvaluationException> {
        let left = self.left.get_value(context)?;
        let right = self.right.get_value(context)?;
        self.operate(&left, &right)
    }

    fn to_string_ast(&self) -> String {
        format!("({} + {})", self.left.to_string_ast(), self.right.to_string_ast())
    }
}

impl BinaryOperator for OpPlus {
    fn left(&self) -> &dyn SpelNode { &*self.left }
    fn right(&self) -> &dyn SpelNode { &*self.right }
    fn operator_name(&self) -> &str { "+" }

    fn operate(&self, left: &TypedValue, right: &TypedValue) -> Result<TypedValue, EvaluationException> {
        match (left.value(), right.value()) {
            (ExpressionValue::Int(l), ExpressionValue::Int(r)) => {
                Ok(TypedValue::new(ExpressionValue::Int(l + r), TypeDescriptor::INT))
            }
            (ExpressionValue::Float(l), ExpressionValue::Float(r)) => {
                Ok(TypedValue::new(ExpressionValue::Float(l + r), TypeDescriptor::FLOAT))
            }
            (ExpressionValue::Int(l), ExpressionValue::Float(r)) => {
                Ok(TypedValue::new(ExpressionValue::Float(*l as f64 + r), TypeDescriptor::FLOAT))
            }
            (ExpressionValue::Float(l), ExpressionValue::Int(r)) => {
                Ok(TypedValue::new(ExpressionValue::Float(l + *r as f64), TypeDescriptor::FLOAT))
            }
            (ExpressionValue::String(l), ExpressionValue::String(r)) => {
                Ok(TypedValue::new(ExpressionValue::String(format!("{l}{r}")), TypeDescriptor::STRING))
            }
            _ => Err(EvaluationException::new("", None, "加法运算不支持的操作数类型")),
        }
    }
}
