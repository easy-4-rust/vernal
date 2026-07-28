//! ParseException — 解析异常基类（对外暴露的）。
//!
//! 对标 Spring `org.springframework.expression.ParseException`。

use thiserror::Error;

use super::expression_exception::ExpressionException;

/// 解析异常（对标 Spring `ParseException`）。
#[derive(Debug, Error)]
pub struct ParseException {
    inner: ExpressionException,
}

impl ParseException {
    /// 创建解析异常。
    pub fn new(
        expression: impl Into<String>,
        position: Option<usize>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            inner: ExpressionException::new(expression, position.map(|p| p as i32), message),
        }
    }

    /// 创建无表达式异常。
    pub fn new_no_expr(message: impl Into<String>) -> Self {
        Self {
            inner: ExpressionException::new_no_expr(None, message),
        }
    }

    /// 获取内部 ExpressionException。
    #[must_use]
    pub fn inner(&self) -> &ExpressionException {
        &self.inner
    }

    /// 简易消息。
    #[must_use]
    pub fn simple_message(&self) -> &str {
        self.inner.simple_message()
    }

    /// 详细消息。
    #[must_use]
    pub fn detailed_string(&self) -> String {
        self.inner.detailed_string()
    }

    /// 表达式字符串（如果有）。
    #[must_use]
    pub fn expression_string(&self) -> Option<&str> {
        self.inner.expression.as_deref()
    }
}

impl std::fmt::Display for ParseException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "解析错误: {}", self.inner)
    }
}
