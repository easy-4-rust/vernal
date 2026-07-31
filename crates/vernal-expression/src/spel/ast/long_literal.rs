//! 长整数字面量。
//!
//! 对标 Spring 的 `LongLiteral`：`42L`

use super::literal::LiteralNode;
use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 长整数字面量节点。
///
/// 对标 Spring 的 `org.springframework.expression.spel.ast.LongLiteral`。
#[derive(Debug, Clone)]
pub struct LongLiteral {
    value: i64,
    original: String,
}

impl LongLiteral {
    /// 创建长整数字面量。
    #[must_use]
    pub fn new(value: i64, original: String) -> Self {
        Self { value, original }
    }
}

impl SpelNode for LongLiteral {
    fn get_value(
        &self,
        _context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        Ok(TypedValue::new(
            ExpressionValue::Int(self.value),
            TypeDescriptor::INT,
        ))
    }

    fn to_string_ast(&self) -> String {
        self.original.clone()
    }
}

impl LiteralNode for LongLiteral {
    fn literal_value(&self) -> &TypedValue {
        unreachable!()
    }
}
