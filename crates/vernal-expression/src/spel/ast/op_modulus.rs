//! 取模运算符。
//!
//! 对标 Spring 的 `OpModulus`：`%`

use crate::typed_value::{TypedValue, ExpressionValue, TypeDescriptor};
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use super::spel_node::SpelNode;
use super::operator::BinaryOperator;

/// 取模运算符节点。

pub struct OpModulus {
    left: Box<dyn SpelNode>,
    right: Box<dyn SpelNode>,
}

impl OpModulus {
    #[must_use]
    pub fn new(left: Box<dyn SpelNode>, right: Box<dyn SpelNode>) -> Self {
        Self { left, right }
    }
}

impl SpelNode for OpModulus {
    fn get_value(&self, context: &dyn EvaluationContext) -> Result<TypedValue, EvaluationException> {
        let left = self.left.get_value(context)?;
        let right = self.right.get_value(context)?;
        self.operate(&left, &right)
    }

    fn to_string_ast(&self) -> String {
        format!("({} % {})", self.left.to_string_ast(), self.right.to_string_ast())
    }
}

impl BinaryOperator for OpModulus {
    fn left(&self) -> &dyn SpelNode { &*self.left }
    fn right(&self) -> &dyn SpelNode { &*self.right }
    fn operator_name(&self) -> &str { "%" }

    fn operate(&self, left: &TypedValue, right: &TypedValue) -> Result<TypedValue, EvaluationException> {
        match (left.value(), right.value()) {
            (ExpressionValue::Int(l), ExpressionValue::Int(r)) => {
                if *r == 0 {
                    return Err(EvaluationException::new("", None, "模零错误"));
                }
                Ok(TypedValue::new(ExpressionValue::Int(l % r), TypeDescriptor::INT))
            }
            _ => Err(EvaluationException::new("", None, "取模运算不支持的操作数类型")),
        }
    }
}
