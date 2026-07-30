//! 赋值表达式节点（对标 Spring `Assign`）。
//!
//! 求值右操作数，将其赋值给左操作数，返回赋值后的值。
//!
//! # Spring 行为
//!
//! - 检查 `EvaluationContext.is_assignment_enabled()`
//! - 左操作数必须是可写的（VariableReference、PropertyOrFieldReference 等）
//! - 通过 `left.getValueRef(state).setValue(value)` 完成赋值
//! - 返回赋值后的值

use super::spel_node::SpelNode;
use super::super::expression_state::ExpressionState;
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
        let mut state = ExpressionState::new(context);
        self.get_value_state(&mut state)
    }

    fn get_value_state(
        &self,
        state: &mut ExpressionState,
    ) -> Result<TypedValue, EvaluationException> {
        // 检查赋值是否启用
        if !state.evaluation_context().is_assignment_enabled() {
            return Err(EvaluationException::new(
                "",
                Some(self.start_position() as i32),
                SpelMessage::VariableAssignmentNotSupported.format_message(&[""]),
            ));
        }

        // 求值右操作数
        let value = self.right.get_value_state(state)?;

        // 检查左操作数是否可写
        if !self.left.is_writable(state.evaluation_context()) {
            return Err(EvaluationException::new(
                "",
                Some(self.left.start_position() as i32),
                SpelMessage::NotAssignable.format_message(&[&self.left.to_string_ast()]),
            ));
        }

        // 写回：通过属性访问器或变量设置
        // 对标 Spring: left.getValueRef(state).setValue(value)
        let target = state.active_context_object().clone();
        let left_str = self.left.to_string_ast();

        // 尝试通过属性访问器写回
        let accessors = state.property_accessors();
        for accessor in &accessors {
            if accessor.can_write(state.evaluation_context(), &target, &left_str) {
                accessor
                    .write(state.evaluation_context(), &target, &left_str, &value)
                    .map_err(|e| {
                        EvaluationException::new(
                            "",
                            None,
                            SpelMessage::ExceptionDuringPropertyWrite
                                .format_message(&[&left_str, &e.to_string()]),
                        )
                    })?;
                return Ok(value);
            }
        }

        // 尝试通过变量设置写回
        state.set_variable(&left_str, value.clone());

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

    #[test]
    fn assign_writes_to_variable() {
        use crate::spel::ast::variable_reference::VariableReference;
        let left = VariableReference::new("myVar".to_string());
        let right = IntLiteral::new(99, "99".to_string());
        let node = Assign::new(Box::new(left), Box::new(right));
        let mut ctx = StandardEvaluationContext::new(TypedValue::null());
        let result = node.get_value(&ctx).unwrap();
        assert_eq!(*result.value(), ExpressionValue::Int(99));
    }
}
