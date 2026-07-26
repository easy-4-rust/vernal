//! 表达式异常。
//!
//! 对标 Spring 的 `ExpressionException`：表达式处理异常的基类。

use std::fmt;

/// 表达式异常基类。
///
/// 对标 Spring 的 `org.springframework.expression.ExpressionException`。
#[derive(Debug, Clone)]
pub struct ExpressionException {
    /// 表达式字符串
    expression: String,
    /// 错误位置
    position: Option<usize>,
    /// 错误消息
    message: String,
}

impl ExpressionException {
    /// 创建表达式异常。
    #[must_use]
    pub fn new(
        expression: impl Into<String>,
        position: Option<usize>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            expression: expression.into(),
            position,
            message: message.into(),
        }
    }

    /// 获取表达式字符串。
    #[must_use]
    pub fn expression(&self) -> &str {
        &self.expression
    }

    /// 获取错误位置。
    #[must_use]
    pub fn position(&self) -> Option<usize> {
        self.position
    }

    /// 获取详细错误信息。
    #[must_use]
    pub fn detailed_string(&self) -> String {
        match self.position {
            Some(pos) => format!("{}: '{}' @ position {}", self.message, self.expression, pos),
            None => format!("{}: '{}'", self.message, self.expression),
        }
    }
}

impl fmt::Display for ExpressionException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.detailed_string())
    }
}

impl std::error::Error for ExpressionException {}
