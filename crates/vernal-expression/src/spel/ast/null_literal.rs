//! 空值字面量。
//!
//! 对标 Spring 的 `NullLiteral`：`null`

use crate::typed_value::{TypedValue, ExpressionValue, TypeDescriptor};
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use super::spel_node::SpelNode;
use super::literal::LiteralNode;

/// 空值字面量节点。
#[derive(Debug, Clone)]
pub struct NullLiteral;

impl NullLiteral {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullLiteral {
    fn default() -> Self {
        Self::new()
    }
}

impl SpelNode for NullLiteral {
    fn get_value(&self, _context: &dyn EvaluationContext) -> Result<TypedValue, EvaluationException> {
        Ok(TypedValue::null())
    }

    fn to_string_ast(&self) -> String {
        "null".to_string()
    }
}

impl LiteralNode for NullLiteral {
    fn literal_value(&self) -> &TypedValue {
        unreachable!()
    }
}
