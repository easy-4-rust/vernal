//! 单精度浮点字面量。
//!
//! 对标 Spring 的 `FloatLiteral`：`3.14f`

use super::literal::LiteralNode;
use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 单精度浮点字面量节点。
#[derive(Debug, Clone)]
pub struct FloatLiteral {
    value: f64,
    original: String,
}

impl FloatLiteral {
    /// 创建单精度浮点字面量。
    #[must_use]
    pub fn new(value: f64, original: String) -> Self {
        Self { value, original }
    }
}

impl SpelNode for FloatLiteral {
    fn get_value(
        &self,
        _context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        Ok(TypedValue::new(
            ExpressionValue::Float(self.value),
            TypeDescriptor::FLOAT,
        ))
    }

    fn to_string_ast(&self) -> String {
        self.original.clone()
    }
}

impl LiteralNode for FloatLiteral {
    fn literal_value(&self) -> &TypedValue {
        unreachable!()
    }
}
