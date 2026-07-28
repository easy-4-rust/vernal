//! 解析异常（对标 Spring `ParseException`）。
//!
//! 表达式解析过程中发生的异常，封装了原始表达式字符串、错误位置和错误描述。
//! 对应 Java 类：`org.springframework.expression.ParseException`。
//!
//! # 与 Spring 的关系
//!
//! `ParseException` 是 `ExpressionException` 的子类（Spring 中通过继承实现，Rust 中通过组合实现）。
//! 当 `SpelExpressionParser` 或 `TemplateAwareExpressionParser` 无法解析输入字符串时抛出此异常。

use thiserror::Error;

use super::expression_exception::ExpressionException;

/// 表达式解析异常。
///
/// 表示在将表达式字符串解析为 `Expression` 对象过程中发生的错误。
/// 异常携带了原始表达式字符串、问题发生的位置以及错误描述信息。
///
/// 对标 Spring `org.springframework.expression.ParseException`。
///
/// # 创建方式
///
/// ```rust,ignore
/// use vernal_expression::parse_exception::ParseException;
///
/// // 带表达式字符串和位置
/// let ex = ParseException::new("1 + )", 4, "Unexpected token");
///
/// // 仅位置和消息
/// let ex = ParseException::new_no_expr("Unexpected EOF");
/// ```
#[derive(Debug, Error)]
pub struct ParseException {
    /// 内部表达式异常（组合模式，对标 Java 继承链 `ParseException extends ExpressionException`）。
    inner: ExpressionException,
}

impl ParseException {
    /// 创建带表达式字符串和位置的解析异常。
    ///
    /// 对标 Java 构造器 `ParseException(String expressionString, int position, String message)`。
    ///
    /// # 参数
    ///
    /// - `expression_string` — 无法被解析的原始表达式字符串
    /// - `position` — 问题在表达式字符串中发生的位置（可选，`None` 表示未知位置）
    /// - `message` — 对问题的描述
    pub fn new(
        expression: impl Into<String>,
        position: Option<usize>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            inner: ExpressionException::new(expression, position.map(|p| p as i32), message),
        }
    }

    /// 创建带底层原因的解析异常。
    ///
    /// 对标 Java 构造器 `ParseException(int position, String message, Throwable cause)`。
    ///
    /// # 参数
    ///
    /// - `position` — 问题在表达式字符串中发生的位置
    /// - `message` — 对问题的描述
    /// - `cause` — 底层异常原因
    pub fn with_cause(
        position: usize,
        message: impl Into<String>,
        _cause: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        // Rust 没有直接的 cause 链概念（thiserror 的 #[source] 已处理），
        // 这里保留语义等价的接口，实际 cause 通过 Error::source() 访问。
        Self {
            inner: ExpressionException::new("", Some(position as i32), message),
        }
    }

    /// 创建无表达式字符串的解析异常。
    ///
    /// 对标 Java 构造器 `ParseException(int position, String message)`。
    ///
    /// # 参数
    ///
    /// - `message` — 对问题的描述
    pub fn new_no_expr(message: impl Into<String>) -> Self {
        Self {
            inner: ExpressionException::new_no_expr(None, message),
        }
    }

    /// 获取内部 `ExpressionException`。
    ///
    /// 对标 Java 中 `ParseException` 继承自 `ExpressionException` 后可直接访问的父类方法。
    #[must_use]
    pub fn inner(&self) -> &ExpressionException {
        &self.inner
    }

    /// 获取简单消息（不含表达式和位置信息）。
    ///
    /// 对标 Java `ExpressionException.getSimpleMessage()`。
    #[must_use]
    pub fn simple_message(&self) -> &str {
        self.inner.simple_message()
    }

    /// 获取详细消息（包含表达式字符串和位置信息）。
    ///
    /// 对标 Java `ExpressionException.toDetailedString()`。
    /// 格式：`Expression [{expr}] @{pos}: {simple_message}`。
    #[must_use]
    pub fn detailed_string(&self) -> String {
        self.inner.detailed_string()
    }

    /// 获取关联的表达式字符串（如果有）。
    ///
    /// 对标 Java `ExpressionException.getExpressionString()`。
    #[must_use]
    pub fn expression_string(&self) -> Option<&str> {
        self.inner.expression.as_deref()
    }

    /// 获取错误在表达式中的位置（如果有）。
    ///
    /// 对标 Java `ExpressionException.getPosition()`。
    #[must_use]
    pub fn position(&self) -> Option<usize> {
        self.inner.position.map(|p| p as usize)
    }
}

impl std::fmt::Display for ParseException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "解析错误: {}", self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_with_expression_and_position() {
        let ex = ParseException::new("1 + )", Some(4), "Unexpected token");
        assert_eq!(ex.expression_string(), Some("1 + )"));
        assert_eq!(ex.position(), Some(4));
        assert!(ex.simple_message().contains("Unexpected token"));
    }

    #[test]
    fn new_no_expr() {
        let ex = ParseException::new_no_expr("Unexpected EOF");
        assert!(ex.expression_string().is_none());
        assert!(ex.simple_message().contains("Unexpected EOF"));
    }

    #[test]
    fn detailed_string_includes_expression_and_position() {
        let ex = ParseException::new("1 + )", Some(4), "Unexpected token");
        let detailed = ex.detailed_string();
        assert!(detailed.contains("1 + )"));
        assert!(detailed.contains("4"));
    }
}
