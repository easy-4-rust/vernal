//! 自增运算符。
//!
//! 对标 Spring 的 `OpInc`：`++`（前缀/后缀）。
//!
//! 前缀 `++a`：先加1再返回新值。
//! 后缀 `a++`：先返回旧值再加1。
//!
//! 写回机制：通过 ExpressionState 的变量/属性访问器写回修改后的值。

use super::spel_node::SpelNode;
use super::super::expression_state::ExpressionState;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::spel::spel_message::SpelMessage;
use crate::typed_value::{ExpressionValue, TypedValue};

/// 自增运算符节点。
pub struct OpInc {
    operand: Box<dyn SpelNode>,
    /// true 表示前缀，false 表示后缀
    prefix: bool,
}

impl OpInc {
    /// 创建自增运算符节点。
    #[must_use]
    pub fn new(operand: Box<dyn SpelNode>, prefix: bool) -> Self {
        Self { operand, prefix }
    }

    /// 计算自增值。
    fn compute_increment(value: &ExpressionValue) -> Result<ExpressionValue, EvaluationException> {
        match value {
            ExpressionValue::Int(i) => Ok(ExpressionValue::Int(i + 1)),
            ExpressionValue::Long(l) => Ok(ExpressionValue::Long(l + 1)),
            ExpressionValue::Float(f) => Ok(ExpressionValue::Float(f + 1.0)),
            ExpressionValue::Double(d) => Ok(ExpressionValue::Double(d + 1.0)),
            _ => Err(EvaluationException::new(
                "",
                None,
                SpelMessage::OperandNotIncrementable.format_message(&[&format!("{:?}", value)]),
            )),
        }
    }
}

impl SpelNode for OpInc {
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
        let value = self.operand.get_value_state(state)?;
        let new_value = Self::compute_increment(value.value())?;
        let new_tv = TypedValue::new(new_value.clone(), value.type_descriptor().clone());

        // 写回：通过属性访问器或变量设置
        // 对标 Spring: operand.getValueRef(state).setValue(newValue)
        self.write_back(state, &new_tv)?;

        // 前缀返回新值，后缀返回旧值
        if self.prefix {
            Ok(new_tv)
        } else {
            Ok(value)
        }
    }

    fn is_writable(&self, _context: &dyn EvaluationContext) -> bool {
        true
    }

    fn to_string_ast(&self) -> String {
        if self.prefix {
            format!("++{}", self.operand.to_string_ast())
        } else {
            format!("{}++", self.operand.to_string_ast())
        }
    }
}

impl OpInc {
    /// 写回修改后的值（对标 Spring ValueRef.setValue）。
    fn write_back(
        &self,
        state: &mut ExpressionState,
        new_value: &TypedValue,
    ) -> Result<(), EvaluationException> {
        // 尝试通过属性访问器写回
        let target = state.active_context_object().clone();
        let accessors = state.property_accessors();
        let operand_str = self.operand.to_string_ast();

        for accessor in &accessors {
            if accessor.can_write(state.evaluation_context(), &target, &operand_str) {
                return accessor
                    .write(state.evaluation_context(), &target, &operand_str, new_value)
                    .map_err(|e| {
                        EvaluationException::new(
                            "",
                            None,
                            SpelMessage::ExceptionDuringPropertyWrite
                                .format_message(&[&operand_str, &e.to_string()]),
                        )
                    });
            }
        }

        // 尝试通过变量设置写回
        state.set_variable(&operand_str, new_value.clone());
        Ok(())
    }
}
