//! ExpressionException — 表达式处理异常基类。
//!
//! 对标 Spring `org.springframework.expression.ExpressionException`，
//! 使用 `thiserror` 派生。

use thiserror::Error;

/// 表达式异常基类（对标 Spring `ExpressionException`）。
#[derive(Debug, Error)]
pub struct ExpressionException {
    /// 关联的表达式字符串。
    pub expression: Option<String>,
    /// 错误位置。
    pub position: Option<i32>,
    /// 简单消息。
    pub simple_message: String,
}

impl ExpressionException {
    /// 创建异常。
    pub fn new(
        expression: impl Into<String>,
        position: Option<i32>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            expression: Some(expression.into()),
            position,
            simple_message: message.into(),
        }
    }

    /// 无表达式的异常。
    pub fn new_no_expr(position: Option<i32>, message: impl Into<String>) -> Self {
        Self {
            expression: None,
            position,
            simple_message: message.into(),
        }
    }

    /// 获取简单消息（对标 Spring `getSimpleMessage()`）。
    #[must_use]
    pub fn simple_message(&self) -> &str {
        &self.simple_message
    }

    /// 详细消息：`Expression [{expr}] @{pos}: {simple}`。
    #[must_use]
    pub fn detailed_string(&self) -> String {
        match (&self.expression, self.position) {
            (Some(expr), Some(pos)) => {
                format!("Expression [{expr}] @{pos}: {}", self.simple_message)
            }
            (Some(expr), None) => {
                format!("Expression [{expr}]: {}", self.simple_message)
            }
            (None, _) => self.simple_message.clone(),
        }
    }
}

impl std::fmt::Display for ExpressionException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.detailed_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_with_expr_and_pos() {
        let e = ExpressionException::new("1+2", Some(3), "boom");
        assert!(e.detailed_string().contains("Expression [1+2] @3: boom"));
    }

    #[test]
    fn build_no_expr() {
        let e = ExpressionException::new_no_expr(None, "boom");
        assert_eq!(e.detailed_string(), "boom");
    }
}
