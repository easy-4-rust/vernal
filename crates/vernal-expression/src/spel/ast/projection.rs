//! 投影运算符节点。
//!
//! 对标 Spring `Projection`：`![]`

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 投影运算符节点。
pub struct Projection {
    expression: Box<dyn SpelNode>,
}

impl Projection {
    /// 创建 Projection 节点。
    ///
    /// # 参数
    ///
    /// - `expression` — 对每个元素应用的表达式
    #[must_use]
    pub fn new(expression: Box<dyn SpelNode>) -> Self {
        Self { expression }
    }

    /// 获取投影表达式引用。
    #[must_use]
    pub fn expression(&self) -> &dyn SpelNode {
        &*self.expression
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
                    TypeDescriptor::OBJECT,
                ))
            }
            _ => Err(EvaluationException::new("", None, "投影运算需要列表操作数")),
        }
    }

    fn child_count(&self) -> usize {
        1
    }

    fn get_child(&self, i: usize) -> Option<&dyn SpelNode> {
        if i == 0 { Some(&*self.expression) } else { None }
    }

    fn start_position(&self) -> usize {
        self.expression.start_position().saturating_sub(2)
    }

    fn end_position(&self) -> usize {
        self.expression.end_position() + 1
    }

    fn to_string_ast(&self) -> String {
        format!("!({})", self.expression.to_string_ast())
    }
}
