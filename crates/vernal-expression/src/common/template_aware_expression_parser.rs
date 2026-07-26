//! 模板感知表达式解析器。
//!
//! 对标 Spring 的 `TemplateAwareExpressionParser`：支持模板语法（`#{expr}`）。

use crate::expression::Expression;
use crate::parse_exception::ParseException;
use crate::parser::ExpressionParser;
use crate::parser_context::ParserContext;
use super::composite_string_expression::CompositeStringExpression;

/// 模板感知表达式解析器。
///
/// 支持模板前缀/后缀（默认 `#{` 和 `}`），将模板分解为字面量和子表达式。
/// 对标 Spring 的 `org.springframework.expression.common.TemplateAwareExpressionParser`。
pub struct TemplateAwareExpressionParser {
    inner: Box<dyn ExpressionParser>,
}

impl TemplateAwareExpressionParser {
    /// 创建模板感知解析器。
    #[must_use]
    pub fn new(inner: Box<dyn ExpressionParser>) -> Self {
        Self { inner }
    }
}

impl ExpressionParser for TemplateAwareExpressionParser {
    fn parse_expression(&self, expression_string: &str) -> Result<Box<dyn Expression>, ParseException> {
        // 默认上下文：标准模板 #{...}
        let context = crate::parser_context::TemplateParserContext::new("#{", "}");
        self.parse_expression_with_context(expression_string, &context)
    }

    fn parse_expression_with_context(
        &self,
        expression_string: &str,
        context: &dyn ParserContext,
    ) -> Result<Box<dyn Expression>, ParseException> {
        if !context.is_template() {
            return self.inner.parse_expression_with_context(expression_string, context);
        }

        let prefix = context.expression_prefix();
        let suffix = context.expression_suffix();

        // 简单模板解析：分割为字面量片段和子表达式
        let mut expressions: Vec<Box<dyn Expression>> = Vec::new();
        let mut current_literal = String::new();
        let mut remaining = expression_string;

        while let Some(prefix_pos) = remaining.find(prefix) {
            current_literal.push_str(&remaining[..prefix_pos]);
            remaining = &remaining[prefix_pos + prefix.len()..];

            if let Some(suffix_pos) = remaining.find(suffix) {
                let sub_expr = &remaining[..suffix_pos];
                let parsed = self.inner.parse_expression(sub_expr)?;
                if !current_literal.is_empty() {
                    expressions.push(Box::new(crate::common::literal_expression::LiteralExpression::new(current_literal.clone())));
                    current_literal.clear();
                }
                expressions.push(parsed);
                remaining = &remaining[suffix_pos + suffix.len()..];
            } else {
                return Err(ParseException::new(
                    expression_string.to_string(),
                    Some(prefix_pos),
                    format!("模板表达式缺少结束符号 '{}'", suffix),
                ));
            }
        }
        current_literal.push_str(remaining);
        if !current_literal.is_empty() {
            expressions.push(Box::new(crate::common::literal_expression::LiteralExpression::new(current_literal)));
        }

        if expressions.len() == 1 {
            Ok(expressions.into_iter().next().unwrap())
        } else {
            Ok(Box::new(CompositeStringExpression::new(
                expression_string.to_string(),
                expressions,
            )))
        }
    }
}
