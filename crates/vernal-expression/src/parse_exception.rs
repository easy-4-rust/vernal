//! 解析异常。
//!
//! 对标 Spring 的 `ParseException`。

use super::expression_exception::ExpressionException;

/// 解析异常。
///
/// 对标 Spring 的 `org.springframework.expression.ParseException`。
#[derive(Debug, Clone)]
pub struct ParseException {
    /// 内部表达式异常
    inner: ExpressionException,
}

impl ParseException {
    /// 创建解析异常。
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

impl std::fmt::Display for ParseException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "解析错误: {}", self.inner)
    }
}

impl std::error::Error for ParseException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}
