//! 投影运算符节点。
//!
//! 对标 Spring 的 `Projection`：`![expression]`

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypedValue};

/// 投影运算符节点。
///
/// 对集合中每个元素应用表达式，返回结果列表。
pub struct Projection {
    expression: Box<dyn SpelNode>,
}

impl Projection {
    #[must_use]
    pub fn new(expression: Box<dyn SpelNode>) -> Self {
        Self { expression }
    }
}

impl SpelNode for Projection {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        let source = context.root_object().clone();
        match source.value() {
            ExpressionValue::List(items) => {
                let mut results = Vec::with_capacity(items.len());
                for _item in items {
                    results.push(self.expression.get_value(context)?);
                }
                Ok(TypedValue::new(
                    ExpressionValue::List(results),
                    crate::typed_value::TypeDescriptor::OBJECT,
                ))
            }
            _ => Err(EvaluationException::new("", None, "投影运算需要列表操作数")),
        }
    }

    fn to_string_ast(&self) -> String {
        format!("!({})", self.expression.to_string_ast())
    }
}
