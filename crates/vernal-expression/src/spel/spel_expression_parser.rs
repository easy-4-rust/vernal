//! SpEL 表达式解析器（对标 Spring `SpelExpressionParser`）。
//!
//! 委托 `InternalSpelExpressionParser` 执行真正的递归下降解析。

use super::internal_spel_expression_parser::InternalSpelExpressionParser;
use crate::expression::Expression;
use crate::parse_exception::ParseException;
use crate::parser::ExpressionParser;
use crate::parser_context::ParserContext;

/// SpEL 表达式解析器（对标 Spring `SpelExpressionParser`）。
///
/// 线程安全、可复用。每次 `parse_expression` 调用内部创建
/// `InternalSpelExpressionParser`（非线程安全，但开销极低）。
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
        let mut parser = InternalSpelExpressionParser::new();
        parser
            .do_parse_expression(expression_string)
            .map_err(|e| ParseException::new(expression_string, e.position, &e.message))
    }

    fn parse_expression_with_context(
        &self,
        expression_string: &str,
        _context: &dyn ParserContext,
    ) -> Result<Box<dyn Expression>, ParseException> {
        self.parse_expression(expression_string)
    }
}
