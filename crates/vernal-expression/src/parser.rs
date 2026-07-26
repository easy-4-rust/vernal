//! 表达式解析器。

/// 表达式解析器 trait。
///
/// 对标 Spring 的 `ExpressionParser`。
pub trait ExpressionParser {
    /// 解析表达式字符串为表达式对象。
    fn parse(&self, expression: &str) -> Result<Box<dyn super::Expression>, ExpressionError>;
}

/// 表达式解析错误。
#[derive(Debug, Clone)]
pub struct ExpressionError {
    pub message: String,
}

impl std::fmt::Display for ExpressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "表达式解析错误: {}", self.message)
    }
}

impl std::error::Error for ExpressionError {}
