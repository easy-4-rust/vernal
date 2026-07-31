//! 安全导航运算符（对标 Spring `SafeNavigation`）。
//!
//! 对标 Java `org.springframework.expression.spel.ast.SafeNavigation`。
//! 支持 `?.` 运算符，当目标为 null 时返回 null 而不是抛出 NPE。
//!
//! # Spring 行为
//!
//! - `a?.b` — 如果 `a` 为 null，返回 null
//! - `a?.b?.c` — 链式安全导航
//! - `a?.method()` — 方法调用安全导航
//! - `a?.[0]` — 索引安全导航
//!
//! # 实现
//!
//! 在 Rust 中，安全导航通过 `Option<T>` 语义实现：
//! - 当目标为 `ExpressionValue::Null` 时，返回 `TypedValue::null()`
//! - 否则正常求值

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::TypedValue;

/// 安全导航运算符节点。
///
/// 对标 Spring `SafeNavigation`。
/// 包装一个子表达式，当子表达式求值结果为 null 时返回 null。
pub struct SafeNavigation {
    /// 被包装的子表达式。
    expression: Box<dyn SpelNode>,
}

impl SafeNavigation {
    /// 创建安全导航节点。
    #[must_use]
    pub fn new(expression: Box<dyn SpelNode>) -> Self {
        Self { expression }
    }

    /// 获取被包装的表达式。
    #[must_use]
    pub fn expression(&self) -> &dyn SpelNode {
        &*self.expression
    }
}

impl SpelNode for SafeNavigation {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        // 求值子表达式
        let result = self.expression.get_value(context)?;

        // 如果结果为 null，返回 null（安全导航语义）
        if result.is_null() {
            return Ok(TypedValue::null());
        }

        // 否则返回原始结果
        Ok(result)
    }

    fn is_writable(&self, context: &dyn EvaluationContext) -> bool {
        // 安全导航的可写性取决于子表达式
        self.expression.is_writable(context)
    }

    fn is_null_safe(&self) -> bool {
        true
    }

    fn start_position(&self) -> usize {
        self.expression.start_position()
    }

    fn end_position(&self) -> usize {
        self.expression.end_position()
    }

    fn to_string_ast(&self) -> String {
        format!("?.{}", self.expression.to_string_ast())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spel::ast::null_literal::NullLiteral;
    use crate::spel::ast::int_literal::IntLiteral;
    use crate::spel::support::standard_evaluation_context::StandardEvaluationContext;
    use crate::typed_value::ExpressionValue;

    #[test]
    fn safe_nav_with_null() {
        let node = SafeNavigation::new(Box::new(NullLiteral::new()));
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let result = node.get_value(&ctx).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn safe_nav_with_value() {
        let node = SafeNavigation::new(Box::new(IntLiteral::new(42, "42".to_string())));
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let result = node.get_value(&ctx).unwrap();
        assert_eq!(*result.value(), ExpressionValue::Int(42));
    }

    #[test]
    fn is_null_safe() {
        let node = SafeNavigation::new(Box::new(NullLiteral::new()));
        assert!(node.is_null_safe());
    }
}
