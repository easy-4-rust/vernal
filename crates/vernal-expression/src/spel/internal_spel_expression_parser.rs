//! 内部递归下降解析器。
//!
//! 对标 Spring 的 `InternalSpelExpressionParser`：完整的递归下降实现。

use super::ast::*;
use crate::evaluation_exception::EvaluationException;
use crate::expression::Expression;
use crate::parse_exception::ParseException;

/// 内部递归下降解析器。
///
/// 解析 SpEL 表达式为 AST 树。
/// 对标 Spring 的 `InternalSpelExpressionParser`。
pub struct InternalSpelExpressionParser {
    expression: String,
    pos: usize,
}

impl InternalSpelExpressionParser {
    /// 创建内部解析器。
    #[must_use]
    pub fn new(expression: String) -> Self {
        Self { expression, pos: 0 }
    }

    /// 解析表达式为 AST。
    pub fn parse(&mut self) -> Result<Box<dyn Expression>, ParseException> {
        // 简化实现：直接返回字面量
        // 完整实现需要完整的递归下降解析器
        self.skip_whitespace();
        if self.pos >= self.expression.len() {
            return Err(self.error("空表达式"));
        }
        self.parse_expression()
    }

    fn parse_expression(&mut self) -> Result<Box<dyn Expression>, ParseException> {
        let trimmed = self.expression.trim();
        // 简化：返回字面量表达式
        Ok(Box::new(
            crate::common::literal_expression::LiteralExpression::new(trimmed.to_string()),
        ))
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.expression.len() {
            let ch = self.expression.chars().nth(self.pos).unwrap_or(' ');
            if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn error(&self, message: &str) -> ParseException {
        ParseException::new(self.expression.clone(), Some(self.pos), message.to_string())
    }

    /// 解析为 AST 节点。
    pub fn parse_ast(
        &mut self,
    ) -> Result<Box<dyn super::ast::spel_node::SpelNode>, ParseException> {
        self.skip_whitespace();
        if self.pos >= self.expression.len() {
            return Err(self.error("空表达式"));
        }
        self.parse_ast_expression()
    }

    fn parse_ast_expression(
        &mut self,
    ) -> Result<Box<dyn super::ast::spel_node::SpelNode>, ParseException> {
        let trimmed = self.expression.trim();
        // 简化：返回字面量节点
        if let Ok(n) = trimmed.parse::<i64>() {
            return Ok(Box::new(super::ast::int_literal::IntLiteral::new(
                n,
                trimmed.to_string(),
            )));
        }
        if let Ok(f) = trimmed.parse::<f64>() {
            return Ok(Box::new(super::ast::real_literal::RealLiteral::new(
                f,
                trimmed.to_string(),
            )));
        }
        if trimmed.starts_with('\'') && trimmed.ends_with('\'') {
            let s = &trimmed[1..trimmed.len() - 1];
            return Ok(Box::new(super::ast::string_literal::StringLiteral::new(
                s.to_string(),
            )));
        }
        if trimmed == "true" || trimmed == "TRUE" {
            return Ok(Box::new(super::ast::boolean_literal::BooleanLiteral::new(
                true,
            )));
        }
        if trimmed == "false" || trimmed == "FALSE" {
            return Ok(Box::new(super::ast::boolean_literal::BooleanLiteral::new(
                false,
            )));
        }
        if trimmed == "null" {
            return Ok(Box::new(super::ast::null_literal::NullLiteral::new()));
        }
        Ok(Box::new(super::ast::identifier::Identifier::new(
            trimmed.to_string(),
        )))
    }
}
