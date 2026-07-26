//! 表达式解析器 trait。
//!
//! 对标 Spring 的 `ExpressionParser` 接口：解析表达式字符串为表达式对象。

use super::expression::Expression;
use super::parse_exception::ParseException;
use super::parser_context::ParserContext;

/// 表达式解析器 trait。
///
/// 解析表达式字符串为 [`Expression`] 对象。
/// 对标 Spring 的 `org.springframework.expression.ExpressionParser`。
///
/// # 实现方式
///
/// - `SpelExpressionParser`：SpEL 解析器
/// - `TemplateAwareExpressionParser`：模板感知解析器
pub trait ExpressionParser: Send + Sync {
    /// 解析表达式字符串。
    fn parse_expression(
        &self,
        expression_string: &str,
    ) -> Result<Box<dyn Expression>, ParseException>;

    /// 使用指定上下文解析表达式字符串。
    fn parse_expression_with_context(
        &self,
        expression_string: &str,
        context: &dyn ParserContext,
    ) -> Result<Box<dyn Expression>, ParseException>;
}
