//! 表达式解析器 trait（对标 Spring `ExpressionParser` 接口）。
//!
//! 将表达式字符串解析为可重复求值的 `Expression` 对象。
//! 对应 Java 接口：`org.springframework.expression.ExpressionParser`。
//!
//! # 实现方式
//!
//! - `SpelExpressionParser`：SpEL 解析器（最常用）
//! - `TemplateAwareExpressionParser`：模板感知解析器

use super::expression::Expression;
use super::parse_exception::ParseException;
use super::parser_context::ParserContext;

/// 表达式解析器 trait（对标 Spring `ExpressionParser` 接口）。
///
/// 将表达式字符串解析为 [`Expression`] 对象，解析后的表达式可被重复求值。
///
/// # 与 Spring 的关系
///
/// Spring `ExpressionParser` 接口定义了两个方法：
/// - `parseExpression(String)` — 解析标准表达式
/// - `parseExpression(String, ParserContext)` — 使用自定义上下文解析（支持模板）
///
/// Rust trait 将这两个方法签名直接映射。
///
/// # 线程安全性
///
/// 实现者必须满足 `Send + Sync`，以支持多线程并发解析（与 Spring `SpelExpressionParser` 线程安全语义一致）。
///
/// # 使用示例
///
/// ```rust,ignore
/// use vernal_expression::spel::spel_expression_parser::SpelExpressionParser;
/// use vernal_expression::*;
///
/// let parser = SpelExpressionParser::new();
/// let expr = parser.parse_expression("1 + 2").unwrap();
/// let ctx = vernal_expression::spel::support::standard_evaluation_context::StandardEvaluationContext::new(TypedValue::null());
/// let result = expr.get_value_with_context(&ctx).unwrap();
/// assert_eq!(*result.value(), ExpressionValue::Int(3));
/// ```
pub trait ExpressionParser: Send + Sync {
    /// 解析表达式字符串并返回可求值的 `Expression` 对象。
    ///
    /// 对标 Java `ExpressionParser.parseExpression(String expressionString)`。
    ///
    /// # 参数
    ///
    /// - `expression_string` — 待解析的原始表达式字符串
    ///
    /// # 返回
    ///
    /// 解析成功的 `Expression` 对象，可被重复求值。
    ///
    /// # 错误
    ///
    /// 如果表达式字符串语法错误，返回 `ParseException`。
    fn parse_expression(
        &self,
        expression_string: &str,
    ) -> Result<Box<dyn Expression>, ParseException>;

    /// 使用指定的解析上下文解析表达式字符串。
    ///
    /// 对标 Java `ExpressionParser.parseExpression(String expressionString, ParserContext context)`。
    ///
    /// # 参数
    ///
    /// - `expression_string` — 待解析的原始表达式字符串
    /// - `context` — 解析上下文，用于控制模板表达式的前缀/后缀等
    ///
    /// # 返回
    ///
    /// 解析成功的 `Expression` 对象。
    ///
    /// # 错误
    ///
    /// 如果表达式字符串语法错误，返回 `ParseException`。
    fn parse_expression_with_context(
        &self,
        expression_string: &str,
        context: &dyn ParserContext,
    ) -> Result<Box<dyn Expression>, ParseException>;
}
