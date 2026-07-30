//! 表达式切点。
//!
//! 对应 spring-aop `ExpressionPointcut`。
//! 使用字符串表达式定义切点。

use crate::Operation;
use crate::pointcut::Pointcut;

/// 表达式切点接口。
///
/// 对应 spring-aop `ExpressionPointcut`。
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::ExpressionPointcut;
///
/// let pointcut = ExpressionPointcut::from_expression("execution(* com.example.*.*(..))");
/// ```
pub trait ExpressionPointcut: Pointcut {
    /// 获取切点表达式。
    fn get_expression(&self) -> Option<&str>;
}

/// 基于字符串的表达式切点。
pub struct StringExpressionPointcut {
    expression: String,
}

impl StringExpressionPointcut {
    /// 创建新的表达式切点。
    pub fn new(expression: impl Into<String>) -> Self {
        Self {
            expression: expression.into(),
        }
    }
}

impl Pointcut for StringExpressionPointcut {
    fn matches(&self, _operation: &Operation) -> bool {
        // 简化实现：始终返回 true
        // 实际实现应该解析表达式并匹配
        true
    }
}

impl ExpressionPointcut for StringExpressionPointcut {
    fn get_expression(&self) -> Option<&str> {
        Some(&self.expression)
    }
}

impl std::fmt::Debug for StringExpressionPointcut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StringExpressionPointcut")
            .field("expression", &self.expression)
            .finish()
    }
}

/// 从字符串创建表达式切点。
impl From<&str> for Box<dyn ExpressionPointcut> {
    fn from(expression: &str) -> Self {
        Box::new(StringExpressionPointcut::new(expression))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_expression_pointcut() {
        let pointcut = StringExpressionPointcut::new("execution(* com.example.*.*(..))");
        assert_eq!(
            pointcut.get_expression(),
            Some("execution(* com.example.*.*(..))")
        );

        let op = Operation::new("com.example.Service", "method");
        assert!(pointcut.matches(&op));
    }
}
