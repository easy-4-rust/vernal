//! SpEL 表达式解析器。
//!
//! 对标 Spring 的 `SpelExpressionParser`。

use super::ast::spel_node::SpelNode;
use super::spel_expression::SpelExpression;
use crate::expression::Expression;
use crate::parse_exception::ParseException;
use crate::parser::ExpressionParser;
use crate::parser_context::ParserContext;

/// SpEL 表达式解析器。
///
/// 对标 Spring 的 `org.springframework.expression.spel.standard.SpelExpressionParser`。
pub struct SpelExpressionParser;

impl SpelExpressionParser {
    /// 创建 SpEL 解析器。
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for SpelExpressionParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionParser for SpelExpressionParser {
    fn parse_expression(
        &self,
        expression_string: &str,
    ) -> Result<Box<dyn Expression>, ParseException> {
        // 简化实现：直接解析为字面量或标识符
        let trimmed = expression_string.trim();

        // 尝试解析为整数字面量
        if let Ok(i) = trimmed.parse::<i64>() {
            let node = super::ast::int_literal::IntLiteral::new(i, trimmed.to_string());
            return Ok(Box::new(SpelExpression::new(
                trimmed.to_string(),
                Box::new(node),
            )));
        }

        // 尝试解析为浮点字面量
        if let Ok(f) = trimmed.parse::<f64>() {
            let node = super::ast::real_literal::RealLiteral::new(f, trimmed.to_string());
            return Ok(Box::new(SpelExpression::new(
                trimmed.to_string(),
                Box::new(node),
            )));
        }

        // 尝试解析为布尔字面量
        match trimmed {
            "true" | "TRUE" => {
                let node = super::ast::boolean_literal::BooleanLiteral::new(true);
                return Ok(Box::new(SpelExpression::new(
                    trimmed.to_string(),
                    Box::new(node),
                )));
            }
            "false" | "FALSE" => {
                let node = super::ast::boolean_literal::BooleanLiteral::new(false);
                return Ok(Box::new(SpelExpression::new(
                    trimmed.to_string(),
                    Box::new(node),
                )));
            }
            "null" => {
                let node = super::ast::null_literal::NullLiteral::new();
                return Ok(Box::new(SpelExpression::new(
                    trimmed.to_string(),
                    Box::new(node),
                )));
            }
            _ => {}
        }

        // 尝试解析为字符串字面量
        if trimmed.starts_with('\'') && trimmed.ends_with('\'') {
            let s = &trimmed[1..trimmed.len() - 1];
            let node = super::ast::string_literal::StringLiteral::new(s.to_string());
            return Ok(Box::new(SpelExpression::new(
                trimmed.to_string(),
                Box::new(node),
            )));
        }

        // 默认解析为标识符
        let node = super::ast::identifier::Identifier::new(trimmed.to_string());
        Ok(Box::new(SpelExpression::new(
            trimmed.to_string(),
            Box::new(node),
        )))
    }

    fn parse_expression_with_context(
        &self,
        expression_string: &str,
        _context: &dyn ParserContext,
    ) -> Result<Box<dyn Expression>, ParseException> {
        self.parse_expression(expression_string)
    }
}
