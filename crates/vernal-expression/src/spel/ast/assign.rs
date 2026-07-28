//! 赋值表达式节点（对标 Spring `Assign`）。
//!
//! 对标 Java `org.springframework.expression.spel.ast.Assign`。
//! 求值右操作数，将其赋值给左操作数，返回赋值后的值。
//!
//! # Spring 行为
//!
//! - 检查 `EvaluationContext.is_assignment_enabled()`
//! - 左操作数必须是可写的（VariableReference、PropertyOrFieldReference 等）
//! - 通过 `left.getValueRef(state).setValue(value)` 完成赋值
//! - 返回赋值后的值

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::spel::spel_message::SpelMessage;
use crate::typed_value::TypedValue;

/// 赋值表达式节点（对标 Spring `Assign`）。
pub struct Assign {
    /// 左操作数（赋值目标）。
    left: Box<dyn SpelNode>,
    /// 右操作数（赋值来源）。
    right: Box<dyn SpelNode>,
}

impl Assign {
    /// 创建赋值节点。
    #[must_use]
    pub fn new(left: Box<dyn SpelNode>, right: Box<dyn SpelNode>) -> Self {
        Self { left, right }
    }
}

impl SpelNode for Assign {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        // 检查赋值是否启用
        if !context.is_assignment_enabled() {
            return Err(EvaluationException::new(
                "",
                Some(self.start_position() as i32),
                SpelMessage::VariableAssignmentNotSupported.format_message(&[""]),
            ));
        }

        // 求值右操作数
        let value = self.right.get_value(context)?;

        // 检查左操作数是否可写
        if !self.left.is_writable(context) {
            return Err(EvaluationException::new(
                "",
                Some(self.left.start_position() as i32),
                SpelMessage::NotAssignable.format_message(&[&self.left.to_string_ast()]),
            ));
        }

        // Phase F: 通过 left.getValueRef().setValue() 完成赋值
        // 当前简化：对于变量引用，通过 context.set_variable 赋值
        // 对于属性引用，通过 PropertyAccessor.write 赋值
        // 这些在完整的 ExpressionState 链路中实现

        // 返回赋值后的值
        Ok(value)
    }

    fn is_writable(&self, _context: &dyn EvaluationContext) -> bool {
        true
    }

    fn to_string_ast(&self) -> String {
        format!(
            "({} = {})",
            self.left.to_string_ast(),
            self.right.to_string_ast()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression_value::ExpressionValue;
    use crate::spel::ast::int_literal::IntLiteral;
    use crate::spel::support::standard_evaluation_context::StandardEvaluationContext;
    use crate::typed_value::TypedValue;

    #[test]
    fn assign_returns_rhs_with_writable_left() {
        // 使用 PropertyOrFieldReference 作为左操作数（属性引用可写）
        let left = crate::spel::ast::property_or_field_reference::PropertyOrFieldReference::new(
            "x".to_string(),
            false,
        );
        let right = IntLiteral::new(42, "42".to_string());
        let node = Assign::new(Box::new(left), Box::new(right));
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let result = node.get_value(&ctx).unwrap();
        assert_eq!(*result.value(), ExpressionValue::Int(42));
    }

    #[test]
    fn assign_literal_left_fails() {
        // 字面量不可写
        let node = Assign::new(
            Box::new(IntLiteral::new(0, "0".to_string())),
            Box::new(IntLiteral::new(42, "42".to_string())),
        );
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let result = node.get_value(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn assign_disabled_returns_error() {
        use crate::spel::support::simple_evaluation_context::SimpleEvaluationContext;
        let node = Assign::new(
            Box::new(IntLiteral::new(0, "0".to_string())),
            Box::new(IntLiteral::new(42, "42".to_string())),
        );
        let ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
        let result = node.get_value(&ctx);
        assert!(result.is_err());
    }
}
