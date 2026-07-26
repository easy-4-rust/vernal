//! 布尔字面量。
//!
//! 对标 Spring 的 `BooleanLiteral`：`TRUE` / `FALSE`

use super::literal::LiteralNode;
use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 布尔字面量节点。
#[derive(Debug, Clone)]
pub struct BooleanLiteral {
    value: bool,
}

impl BooleanLiteral {
    #[must_use]
    pub fn new(value: bool) -> Self {
        Self { value }
    }
}

impl SpelNode for BooleanLiteral {
    fn get_value(
        &self,
        _context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        Ok(TypedValue::new(
            ExpressionValue::Boolean(self.value),
            TypeDescriptor::BOOLEAN,
        ))
    }

    fn to_string_ast(&self) -> String {
        if self.value {
            "TRUE".to_string()
        } else {
            "FALSE".to_string()
        }
    }
}

impl LiteralNode for BooleanLiteral {
    fn literal_value(&self) -> &TypedValue {
        unreachable!()
    }
}
