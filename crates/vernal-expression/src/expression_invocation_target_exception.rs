//! 方法调用目标异常。
//!
//! 对标 Spring 的 `ExpressionInvocationTargetException`：包装被调用方法抛出的异常。

use crate::evaluation_exception::EvaluationException;

/// 方法调用目标异常。
///
/// 对标 Spring 的 `org.springframework.expression.ExpressionInvocationTargetException`。
#[derive(Debug, Clone)]
pub struct ExpressionInvocationTargetException {
    /// 原始异常
    cause: String,
}

impl ExpressionInvocationTargetException {
    /// 创建方法调用目标异常。
    #[must_use]
    pub fn new(cause: String) -> Self {
        Self { cause }
    }

    /// 获取原始异常信息。
    #[must_use]
    pub fn cause(&self) -> &str {
        &self.cause
    }

    /// 转换为公共 EvaluationException。
    #[must_use]
    pub fn to_evaluation_exception(&self) -> EvaluationException {
        EvaluationException::new("", None, format!("方法调用失败: {}", self.cause))
    }
}

impl std::fmt::Display for ExpressionInvocationTargetException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "方法调用目标异常: {}", self.cause)
    }
}

impl std::error::Error for ExpressionInvocationTargetException {}
