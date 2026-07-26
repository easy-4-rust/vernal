//! 字符串字面量。
//!
//! 对标 Spring 的 `StringLiteral`：`'hello'`

use crate::typed_value::{TypedValue, ExpressionValue, TypeDescriptor};
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use super::spel_node::SpelNode;
use super::literal::LiteralNode;

/// 字符串字面量节点。
#[derive(Debug, Clone)]
pub struct StringLiteral {
    value: String,
}

impl StringLiteral {
    #[must_use]
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

impl SpelNode for StringLiteral {
    fn get_value(&self, _context: &dyn EvaluationContext) -> Result<TypedValue, EvaluationException> {
        Ok(TypedValue::new(ExpressionValue::String(self.value.clone()), TypeDescriptor::STRING))
    }

    fn to_string_ast(&self) -> String {
        format!("'{}'", self.value)
    }
}

impl LiteralNode for StringLiteral {
    fn literal_value(&self) -> &TypedValue {
        unreachable!()
    }
}
