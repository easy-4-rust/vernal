//! 投影运算符节点。
//!
//! 对标 Spring `Projection`：`![expression]`。
//!
//! # Spring 求值语义
//!
//! Spring 的 `Projection` 对集合中每个元素执行表达式，收集结果。
//! 每个元素被 push 到 `ExpressionState.activeContextObject` 栈，
//! 使表达式中的 `#this` 引用当前元素。
//!
//! 本实现通过 `get_value_state` 操作 `ExpressionState` 的 active context 栈，
//! 实现与 Spring 完全一致的元素上下文替换。

use super::super::expression_state::ExpressionState;
use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 投影运算符节点。
pub struct Projection {
    expression: Box<dyn SpelNode>,
    null_safe: bool,
}

impl Projection {
    /// 创建 Projection 节点。
    #[must_use]
    pub fn new(expression: Box<dyn SpelNode>) -> Self {
        Self {
            expression,
            null_safe: false,
        }
    }

    /// 创建 null-safe Projection 节点。
    #[must_use]
    pub fn new_null_safe(expression: Box<dyn SpelNode>) -> Self {
        Self {
            expression,
            null_safe: true,
        }
    }

    /// 获取投影表达式引用。
    #[must_use]
    pub fn expression(&self) -> &dyn SpelNode {
        &*self.expression
    }

    /// 对列表执行投影操作。
    fn project_list(
        &self,
        items: &[TypedValue],
        state: &mut ExpressionState,
    ) -> Result<TypedValue, EvaluationException> {
        let mut results = Vec::with_capacity(items.len());

        for item in items {
            // 对标 Spring: push element as active context, enter scope
            state.push_active_context_object(item.clone());
            state.enter_scope();

            // 在当前元素上下文中求值表达式
            let result = self.expression.get_value_state(state)?;

            // 对标 Spring: pop context, exit scope
            state.exit_scope();
            state.pop_active_context_object();

            results.push(result);
        }

        Ok(TypedValue::new(
            ExpressionValue::List(results),
            TypeDescriptor::OBJECT,
        ))
    }

    /// 对 Map 执行投影操作。
    fn project_map(
        &self,
        entries: &[(TypedValue, TypedValue)],
        state: &mut ExpressionState,
    ) -> Result<TypedValue, EvaluationException> {
        let mut results = Vec::with_capacity(entries.len());

        for (key, value) in entries {
            // 对标 Spring: push entry as active context
            let entry = TypedValue::new(
                ExpressionValue::Map(vec![(key.clone(), value.clone())]),
                TypeDescriptor::from_type_name("Map"),
            );
            state.push_active_context_object(entry);
            state.enter_scope();

            let result = self.expression.get_value_state(state)?;

            state.exit_scope();
            state.pop_active_context_object();

            results.push(result);
        }

        Ok(TypedValue::new(
            ExpressionValue::List(results),
            TypeDescriptor::OBJECT,
        ))
    }
}

impl SpelNode for Projection {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        let mut state = ExpressionState::new(context);
        self.get_value_state(&mut state)
    }

    fn is_null_safe(&self) -> bool {
        self.null_safe
    }

    fn get_value_state(
        &self,
        state: &mut ExpressionState,
    ) -> Result<TypedValue, EvaluationException> {
        let source = state.active_context_object().clone();

        // null-safe 检查（对标 Spring Projection null-safe）
        if self.null_safe && source.is_null() {
            return Ok(TypedValue::null());
        }

        match source.value() {
            ExpressionValue::List(items) => self.project_list(items, state),
            ExpressionValue::Map(entries) => self.project_map(entries, state),
            _ => Err(EvaluationException::new(
                "",
                None,
                "投影运算需要列表或映射操作数",
            )),
        }
    }

    fn child_count(&self) -> usize {
        1
    }

    fn get_child(&self, i: usize) -> Option<&dyn SpelNode> {
        if i == 0 {
            Some(&*self.expression)
        } else {
            None
        }
    }

    fn start_position(&self) -> usize {
        self.expression.start_position().saturating_sub(2)
    }

    fn end_position(&self) -> usize {
        self.expression.end_position() + 1
    }

    fn to_string_ast(&self) -> String {
        format!("!({})", self.expression.to_string_ast())
    }
}
