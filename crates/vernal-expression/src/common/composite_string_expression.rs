//! 模板组合表达式。
//!
//! 对标 Spring 的 `CompositeStringExpression`：模板表达式分解为多个子表达式。

use crate::typed_value::{TypedValue, ExpressionValue, TypeDescriptor};
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::expression::Expression;

/// 模板组合表达式。
///
/// 将模板分解为字面量片段和真实表达式，结果按顺序连接。
/// 对标 Spring 的 `org.springframework.expression.common.CompositeStringExpression`。
pub struct CompositeStringExpression {
    expression_string: String,
    expressions: Vec<Box<dyn Expression>>,
}

impl CompositeStringExpression {
    /// 创建模板组合表达式。
    #[must_use]
    pub fn new(expression_string: String, expressions: Vec<Box<dyn Expression>>) -> Self {
        Self { expression_string, expressions }
    }

    /// 获取子表达式列表。
    #[must_use]
    pub fn expressions(&self) -> &[Box<dyn Expression>] {
        &self.expressions
    }
}

impl Expression for CompositeStringExpression {
    fn expression_string(&self) -> &str {
        &self.expression_string
    }

    fn get_value(&self) -> Result<TypedValue, EvaluationException> {
        Err(EvaluationException::new(&self.expression_string, None, "模板表达式需要指定上下文"))
    }

    fn get_value_with_context(&self, context: &dyn EvaluationContext) -> Result<TypedValue, EvaluationException> {
        let mut result = String::new();
        for expr in &self.expressions {
            let value = expr.get_value_with_context(context)?;
            match value.value() {
                ExpressionValue::String(s) => result.push_str(s),
                _ => {
                    return Err(EvaluationException::new(
                        &self.expression_string,
                        None,
                        "模板表达式中所有子表达式必须返回字符串",
                    ));
                }
            }
        }
        Ok(TypedValue::new(ExpressionValue::String(result), TypeDescriptor::STRING))
    }

    fn get_value_with_root(&self, context: &dyn EvaluationContext, _root: &TypedValue) -> Result<TypedValue, EvaluationException> {
        self.get_value_with_context(context)
    }
}
