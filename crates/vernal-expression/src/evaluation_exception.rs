//! EvaluationException — 求值阶段异常基类。
//!
//! 对标 Spring `org.springframework.expression.EvaluationException`。
//! 包装内部 `ExpressionException`。

use thiserror::Error;

use super::expression_exception::ExpressionException;

/// 求值异常（对标 Spring `EvaluationException`）。
#[derive(Debug, Error)]
pub struct EvaluationException {
    /// 内部表达式异常。
    inner: ExpressionException,
}

impl EvaluationException {
    /// 创建求值异常。
    pub fn new(
        expression: impl Into<String>,
        position: Option<i32>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            inner: ExpressionException::new(expression, position, message),
        }
    }

    /// 创建无表达式异常。
    pub fn new_no_expr(message: impl Into<String>) -> Self {
        Self {
            inner: ExpressionException::new_no_expr(None, message),
        }
    }

    /// 获取内部 `ExpressionException`。
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

    /// 位置。
    #[must_use]
    pub fn position(&self) -> Option<i32> {
        self.inner.position
    }
}

impl std::fmt::Display for EvaluationException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "求值错误: {}", self.inner)
    }
}
