//! 减法运算符。
//!
//! 对标 Spring 的 `OpMinus`：`-`（减法、取负）

use super::operator::BinaryOperator;
use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 减法运算符节点。

pub struct OpMinus {
    left: Box<dyn SpelNode>,
    right: Box<dyn SpelNode>,
}

impl OpMinus {
    /// 创建减法运算符节点。
    #[must_use]
    pub fn new(left: Box<dyn SpelNode>, right: Box<dyn SpelNode>) -> Self {
        Self { left, right }
    }
}

impl SpelNode for OpMinus {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        let left = self.left.get_value(context)?;
        let right = self.right.get_value(context)?;
        self.operate(&left, &right)
    }

    fn to_string_ast(&self) -> String {
        format!(
            "({} - {})",
            self.left.to_string_ast(),
            self.right.to_string_ast()
        )
    }
}

impl BinaryOperator for OpMinus {
    fn left(&self) -> &dyn SpelNode {
        &*self.left
    }
    fn right(&self) -> &dyn SpelNode {
        &*self.right
    }
    fn operator_name(&self) -> &str {
        "-"
    }

    fn operate(
        &self,
        left: &TypedValue,
        right: &TypedValue,
    ) -> Result<TypedValue, EvaluationException> {
        match (left.value(), right.value()) {
            (ExpressionValue::Int(l), ExpressionValue::Int(r)) => Ok(TypedValue::new(
                ExpressionValue::Int(l - r),
                TypeDescriptor::INT,
            )),
            (ExpressionValue::Float(l), ExpressionValue::Float(r)) => Ok(TypedValue::new(
                ExpressionValue::Float(l - r),
                TypeDescriptor::FLOAT,
            )),
            _ => Err(EvaluationException::new(
                "",
                None,
                "减法运算不支持的操作数类型",
            )),
        }
    }
}
