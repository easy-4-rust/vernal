//! 表达式 trait。

use super::context::EvaluationContext;

/// 表达式 trait。
///
/// 对标 Spring 的 `Expression`。
pub trait Expression: Send + Sync {
    /// 在给定上下文中求值。
    fn evaluate(&self, context: &dyn EvaluationContext) -> Result<ExpressionValue, ExpressionError>;

    /// 获取表达式字符串。
    fn expression_string(&self) -> &str;
}

/// 表达式值。
#[derive(Debug, Clone)]
pub enum ExpressionValue {
    /// 字符串值
    String(String),
    /// 数字值
    Number(f64),
    /// 布尔值
    Boolean(bool),
    /// 空值
    Null,
}

/// 表达式错误。
#[derive(Debug, Clone)]
pub struct ExpressionError {
    pub message: String,
}

impl std::fmt::Display for ExpressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "表达式错误: {}", self.message)
    }
}

impl std::error::Error for ExpressionError {}
