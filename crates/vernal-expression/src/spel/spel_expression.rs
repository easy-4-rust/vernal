//! SpEL 表达式实现。
//!
//! 对标 Spring 的 `SpelExpression`：已解析的 SpEL 表达式。

use super::ast::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::expression::Expression;
use crate::typed_value::TypedValue;

/// SpEL 表达式。
///
/// 对标 Spring 的 `org.springframework.expression.spel.standard.SpelExpression`。
pub struct SpelExpression {
    /// 原始表达式字符串
    expression_string: String,
    /// AST 根节点
    ast: Box<dyn SpelNode>,
}

impl SpelExpression {
    /// 创建 SpEL 表达式。
    #[must_use]
    pub fn new(expression_string: String, ast: Box<dyn SpelNode>) -> Self {
        Self {
            expression_string,
            ast,
        }
    }

    /// 获取 AST 根节点。
    #[must_use]
    pub fn ast(&self) -> &dyn SpelNode {
        &*self.ast
    }
}

impl Expression for SpelExpression {
    fn expression_string(&self) -> &str {
        &self.expression_string
    }

    fn get_value(&self) -> Result<TypedValue, EvaluationException> {
        // 使用默认上下文求值
        Err(EvaluationException::new(
            &self.expression_string,
            None,
            "需要指定求值上下文",
        ))
    }

    fn get_value_with_context(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        self.ast.get_value(context)
    }

    fn get_value_with_root(
        &self,
        context: &dyn EvaluationContext,
        _root: &TypedValue,
    ) -> Result<TypedValue, EvaluationException> {
        self.ast.get_value(context)
    }
}
