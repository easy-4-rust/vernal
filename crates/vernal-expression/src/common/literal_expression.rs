//! 字面量表达式。
//!
//! 对标 Spring 的 `LiteralExpression`。

use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::expression::Expression;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 字面量表达式。
///
/// 硬编码的字符串字面量，用于模板表达式。
/// 对标 Spring 的 `org.springframework.expression.common.LiteralExpression`。
pub struct LiteralExpression {
    value: String,
}

impl LiteralExpression {
    /// 创建字面量表达式。
    #[must_use]
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

impl Expression for LiteralExpression {
    fn expression_string(&self) -> &str {
        &self.value
    }

    fn get_value(&self) -> Result<TypedValue, EvaluationException> {
        Ok(TypedValue::new(
            ExpressionValue::String(self.value.clone()),
            TypeDescriptor::STRING,
        ))
    }

    fn get_value_with_context(
        &self,
        _context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        self.get_value()
    }

    fn get_value_with_root(
        &self,
        _context: &dyn EvaluationContext,
        _root: &TypedValue,
    ) -> Result<TypedValue, EvaluationException> {
        self.get_value()
    }
}
