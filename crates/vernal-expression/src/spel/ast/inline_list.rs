//! 内联列表节点。
//!
//! 对标 Spring 的 `InlineList`：`{1, 2, 3}`

use crate::typed_value::{TypedValue, ExpressionValue, TypeDescriptor};
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use super::spel_node::SpelNode;

/// 内联列表节点。
pub struct InlineList {
    elements: Vec<Box<dyn SpelNode>>,
}

impl InlineList {
    #[must_use]
    pub fn new(elements: Vec<Box<dyn SpelNode>>) -> Self {
        Self { elements }
    }
}

impl SpelNode for InlineList {
    fn get_value(&self, context: &dyn EvaluationContext) -> Result<TypedValue, EvaluationException> {
        let mut values = Vec::with_capacity(self.elements.len());
        for element in &self.elements {
            values.push(element.get_value(context)?);
        }
        Ok(TypedValue::new(ExpressionValue::List(values), TypeDescriptor::OBJECT))
    }

    fn child_count(&self) -> usize {
        self.elements.len()
    }

    fn to_string_ast(&self) -> String {
        let items: Vec<_> = self.elements.iter().map(|e| e.to_string_ast()).collect();
        format!("{{{}}}", items.join(", "))
    }
}
