//! BeanExpressionException — 对应 Spring `org.springframework.beans.factory.BeanExpressionException`。
//!
//! Bean 表达式求值失败时抛出的异常。

use std::fmt;

/// Bean 表达式求值失败时抛出的异常。
///
/// 对应 Java 类：`org.springframework.beans.factory.BeanExpressionException`。
#[derive(Debug)]
pub struct BeanExpressionException {
    message: String,
    expression: Option<String>,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl BeanExpressionException {
    /// 创建一个新的 BeanExpressionException。
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            expression: None,
            source: None,
        }
    }

    /// 创建一个带有表达式的 BeanExpressionException。
    pub fn with_expression(expression: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            expression: Some(expression.into()),
            source: None,
        }
    }

    /// 创建一个带有原因的 BeanExpressionException。
    pub fn with_cause(
        message: impl Into<String>,
        cause: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            expression: None,
            source: Some(Box::new(cause)),
        }
    }

    /// 获取表达式。
    pub fn expression(&self) -> Option<&str> {
        self.expression.as_deref()
    }

    /// 获取异常消息。
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for BeanExpressionException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref expr) = self.expression {
            write!(f, "Expression parsing failed for expression '{}': {}", expr, self.message)
        } else {
            write!(f, "Bean expression error: {}", self.message)
        }
    }
}

impl std::error::Error for BeanExpressionException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_new() {
        let e = BeanExpressionException::new("parse error");
        assert_eq!(e.message(), "parse error");
        assert!(e.expression().is_none());
    }

    #[test]
    fn test_with_expression() {
        let e = BeanExpressionException::with_expression("#{bean.name}", "evaluation failed");
        assert_eq!(e.expression(), Some("#{bean.name}"));
    }

    #[test]
    fn test_with_cause() {
        let cause = std::io::Error::new(std::io::ErrorKind::Other, "inner");
        let e = BeanExpressionException::with_cause("outer", cause);
        assert!(e.source().is_some());
    }

    #[test]
    fn test_display() {
        let e = BeanExpressionException::with_expression("#{1/0}", "division by zero");
        assert!(format!("{}", e).contains("#{1/0}"));
    }
}
