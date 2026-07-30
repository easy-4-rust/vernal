//! SpEL 表达式解析器（对标 Spring `SpelExpressionParser`）。
//!
//! 委托 `InternalSpelExpressionParser` 执行真正的递归下降解析。
//! 支持模板表达式（如 `Hello #{name}!`）。

use super::internal_spel_expression_parser::InternalSpelExpressionParser;
use crate::common::composite_string_expression::CompositeStringExpression;
use crate::common::literal_expression::LiteralExpression;
use crate::expression::Expression;
use crate::parse_exception::ParseException;
use crate::parser::ExpressionParser;
use crate::parser_context::ParserContext;

/// SpEL 表达式解析器（对标 Spring `SpelExpressionParser`）。
///
/// 线程安全、可复用。每次 `parse_expression` 调用内部创建
/// `InternalSpelExpressionParser`（非线程安全，但开销极低）。
///
/// 支持模板表达式解析（`#{...}`）通过 `parse_expression_with_context`。
pub struct SpelExpressionParser;

impl SpelExpressionParser {
    /// 创建 SpEL 解析器。
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// 解析单个 SpEL 表达式（非模板）。
    fn do_parse_spel_expression(
        &self,
        expression_string: &str,
    ) -> Result<Box<dyn Expression>, ParseException> {
        let mut parser = InternalSpelExpressionParser::new();
        parser
            .do_parse_expression(expression_string)
            .map_err(|e| ParseException::new(expression_string, e.position, &e.message))
    }

    /// 解析模板表达式（对标 Spring `TemplateAwareExpressionParser.parseTemplate`）。
    ///
    /// 扫描 `prefix`/`suffix` 模式，将模板分解为字面量和 SpEL 表达式。
    fn do_parse_template(
        &self,
        expression_string: &str,
        context: &dyn ParserContext,
    ) -> Result<Box<dyn Expression>, ParseException> {
        let prefix = context.expression_prefix();
        let suffix = context.expression_suffix();
        let mut expressions: Vec<Box<dyn Expression>> = Vec::new();
        let mut pos = 0;
        let chars: Vec<char> = expression_string.chars().collect();

        loop {
            // 查找下一个前缀
            let prefix_start = find_subsequence(&chars[pos..], &prefix.chars().collect::<Vec<_>>());
            match prefix_start {
                Some(start) => {
                    // 前缀之前的字面量
                    if start > 0 {
                        let literal: String = chars[pos..pos + start].iter().collect();
                        expressions.push(Box::new(LiteralExpression::new(literal)));
                    }

                    // 查找后缀
                    let expr_start = pos + start + prefix.len();
                    let suffix_offset =
                        find_subsequence(&chars[expr_start..], &suffix.chars().collect::<Vec<_>>());
                    match suffix_offset {
                        Some(end) => {
                            // 提取 SpEL 表达式
                            let spel_expr: String =
                                chars[expr_start..expr_start + end].iter().collect();
                            let expr = self.do_parse_spel_expression(&spel_expr)?;
                            expressions.push(expr);
                            pos = expr_start + end + suffix.len();
                        }
                        None => {
                            return Err(ParseException::new(
                                expression_string,
                                Some(expr_start),
                                &format!("模板表达式中未找到后缀 '{}'", suffix),
                            ));
                        }
                    }
                }
                None => {
                    // 剩余部分作为字面量
                    if pos < chars.len() {
                        let literal: String = chars[pos..].iter().collect();
                        expressions.push(Box::new(LiteralExpression::new(literal)));
                    }
                    break;
                }
            }
        }

        // 如果只有一个表达式且是 SpEL 表达式，直接返回
        if expressions.len() == 1 {
            return Ok(expressions.into_iter().next().unwrap());
        }

        Ok(Box::new(CompositeStringExpression::new(
            expression_string.to_string(),
            expressions,
        )))
    }
}

/// 在字符切片中查找子序列。
fn find_subsequence(haystack: &[char], needle: &[char]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
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
        self.do_parse_spel_expression(expression_string)
    }

    fn parse_expression_with_context(
        &self,
        expression_string: &str,
        context: &dyn ParserContext,
    ) -> Result<Box<dyn Expression>, ParseException> {
        if context.is_template() {
            self.do_parse_template(expression_string, context)
        } else {
            self.do_parse_spel_expression(expression_string)
        }
    }
}
