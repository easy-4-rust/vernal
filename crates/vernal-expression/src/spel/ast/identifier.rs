//! 标识符节点。
//!
//! 对标 Spring 的 `Identifier`：标识符字符串。

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 标识符节点。
pub struct Identifier {
    name: String,
}

impl Identifier {
    #[must_use]
    pub fn new(name: String) -> Self {
        Self { name }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl SpelNode for Identifier {
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
