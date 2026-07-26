//! 限定标识符节点。
//!
//! 对标 Spring 的 `QualifiedIdentifier`：`com.example.Foo`

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 限定标识符节点。
pub struct QualifiedIdentifier {
    name: String,
}

impl QualifiedIdentifier {
    #[must_use]
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl SpelNode for QualifiedIdentifier {
    fn get_value(
        &self,
        _context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        Ok(TypedValue::new(
            ExpressionValue::String(self.name.clone()),
            TypeDescriptor::STRING,
        ))
    }

    fn to_string_ast(&self) -> String {
        self.name.clone()
    }
}
