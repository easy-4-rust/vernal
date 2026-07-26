//! 内联映射节点。
//!
//! 对标 Spring 的 `InlineMap`：`{k1: v1, k2: v2}`

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 内联映射节点。
pub struct InlineMap {
    keys: Vec<Box<dyn SpelNode>>,
    values: Vec<Box<dyn SpelNode>>,
}

impl InlineMap {
    #[must_use]
    pub fn new(keys: Vec<Box<dyn SpelNode>>, values: Vec<Box<dyn SpelNode>>) -> Self {
        Self { keys, values }
    }
}

impl SpelNode for InlineMap {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        let mut pairs = Vec::with_capacity(self.keys.len());
        for (key, value) in self.keys.iter().zip(self.values.iter()) {
            pairs.push((key.get_value(context)?, value.get_value(context)?));
        }
        Ok(TypedValue::new(
            ExpressionValue::Map(pairs),
            TypeDescriptor::OBJECT,
        ))
    }

    fn child_count(&self) -> usize {
        self.keys.len() + self.values.len()
    }

    fn to_string_ast(&self) -> String {
        let entries: Vec<_> = self
            .keys
            .iter()
            .zip(self.values.iter())
            .map(|(k, v)| format!("{}: {}", k.to_string_ast(), v.to_string_ast()))
            .collect();
        format!("{{{}}}", entries.join(", "))
    }
}
