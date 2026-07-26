//! 运算符基类。
//!
//! 对标 Spring 的 `Operator` 抽象类。

use crate::typed_value::{TypedValue, ExpressionValue};
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use super::spel_node::SpelNode;

/// 二元运算符 trait。
///
/// 对标 Spring 的 `org.springframework.expression.spel.ast.Operator`。
pub trait BinaryOperator: SpelNode {
    /// 获取左操作数。
    fn left(&self) -> &dyn SpelNode;

    /// 获取右操作数。
    fn right(&self) -> &dyn SpelNode;

    /// 获取运算符名称。
    fn operator_name(&self) -> &str;

    /// 执行运算。
    fn operate(&self, left: &TypedValue, right: &TypedValue) -> Result<TypedValue, EvaluationException>;
}
