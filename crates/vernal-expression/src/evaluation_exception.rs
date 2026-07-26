//! 求值异常。
//!
//! 对标 Spring 的 `EvaluationException`。

use super::expression_exception::ExpressionException;

/// 求值异常。
///
/// 对标 Spring 的 `org.springframework.expression.EvaluationException`。
#[derive(Debug, Clone)]
pub struct EvaluationException {
    /// 内部表达式异常
    inner: ExpressionException,
}

impl EvaluationException {
    /// 创建求值异常。
    #[must_use]
    pub fn new(
        expression: impl Into<String>,
        position: Option<usize>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            inner: ExpressionException::new(expression, position, message),
        }
    }

    /// 获取内部表达式异常。
    #[must_use]
    pub fn inner(&self) -> &ExpressionException {
        &self.inner
    }
}

impl std::fmt::Display for EvaluationException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "求值错误: {}", self.inner)
    }
}

impl std::error::Error for EvaluationException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}
