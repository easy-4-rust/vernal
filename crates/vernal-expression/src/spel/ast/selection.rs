//! 选择/过滤运算符节点。
//!
//! 对标 Spring `Selection`：`?[criteria]` / `^[criteria]` / `$[criteria]`。
//!
//! # Spring 求值语义
//!
//! Spring 的 `Selection` 对 root context 中的集合求值，把每个元素 push 到
//! `ExpressionState.activeContextObject` 栈，让 criteria 在每个元素上下文中求值。
//! 这使得 `#this` 在 criteria 中引用当前元素。
//!
//! 本实现通过 `get_value_state` 操作 `ExpressionState` 的 active context 栈，
//! 实现与 Spring 完全一致的元素上下文替换。

use super::spel_node::SpelNode;
use super::super::expression_state::ExpressionState;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::spel::spel_message::SpelMessage;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 选择变体（对标 Spring `SELECT` / `SELECT_FIRST` / `SELECT_LAST`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelectionVariant {
    /// 所有匹配元素
    All,
    /// 第一个匹配元素
    First,
    /// 最后一个匹配元素
    Last,
}

/// 选择/过滤运算符节点。
pub struct Selection {
    criteria: Box<dyn SpelNode>,
    variant: SelectionVariant,
    null_safe: bool,
}

impl Selection {
    /// 创建 Selection 节点。
    #[must_use]
    pub fn new(criteria: Box<dyn SpelNode>, variant: SelectionVariant) -> Self {
        Self { criteria, variant, null_safe: false }
    }

    /// 创建 null-safe Selection 节点。
    #[must_use]
    pub fn new_null_safe(criteria: Box<dyn SpelNode>, variant: SelectionVariant) -> Self {
        Self { criteria, variant, null_safe: true }
    }

    /// 获取过滤条件引用。
    #[must_use]
    pub fn criteria(&self) -> &dyn SpelNode {
        &*self.criteria
    }

    /// 获取选择变体。
    #[must_use]
    pub fn variant(&self) -> SelectionVariant {
        self.variant
    }

    /// 对列表执行选择操作（对标 Spring Selection 对 List 的处理）。
    fn select_list(
        &self,
        items: &[TypedValue],
        state: &mut ExpressionState,
    ) -> Result<TypedValue, EvaluationException> {
        let mut results = Vec::new();

        for item in items {
            // 对标 Spring: push element as active context, enter scope
            state.push_active_context_object(item.clone());
            state.enter_scope();

            // 在当前元素上下文中求值 criteria
            let matches = self.criteria.get_value_state(state)?;

            // 对标 Spring: 验证 criteria 结果必须是 Boolean
            let is_match = match matches.value() {
                ExpressionValue::Boolean(b) => *b,
                _ => {
                    return Err(EvaluationException::new(
                        "",
                        None,
                        "选择条件的结果必须是布尔类型",
                    ));
                }
            };

            // 对标 Spring: pop context, exit scope
            state.exit_scope();
            state.pop_active_context_object();

            if is_match {
                results.push(item.clone());
            }
        }

        match self.variant {
            SelectionVariant::All => Ok(TypedValue::new(
                ExpressionValue::List(results),
                TypeDescriptor::OBJECT,
            )),
            SelectionVariant::First => Ok(results.into_iter().next().unwrap_or(TypedValue::null())),
            SelectionVariant::Last => Ok(results.into_iter().last().unwrap_or(TypedValue::null())),
        }
    }

    /// 对 Map 执行选择操作（对标 Spring Selection 对 Map 的处理）。
    fn select_map(
        &self,
        entries: &[(TypedValue, TypedValue)],
        state: &mut ExpressionState,
    ) -> Result<TypedValue, EvaluationException> {
        let mut results = Vec::new();

        for (key, value) in entries {
            // 对标 Spring: push entry as active context
            let entry = TypedValue::new(
                ExpressionValue::Map(vec![(key.clone(), value.clone())]),
                TypeDescriptor::from_type_name("Map"),
            );
            state.push_active_context_object(entry);
            state.enter_scope();

            let matches = self.criteria.get_value_state(state)?;

            let is_match = match matches.value() {
                ExpressionValue::Boolean(b) => *b,
                _ => {
                    return Err(EvaluationException::new(
                        "",
                        None,
                        "选择条件的结果必须是布尔类型",
                    ));
                }
            };

            state.exit_scope();
            state.pop_active_context_object();

            if is_match {
                results.push((key.clone(), value.clone()));
            }
        }

        match self.variant {
            SelectionVariant::All => Ok(TypedValue::new(
                ExpressionValue::Map(results),
                TypeDescriptor::from_type_name("Map"),
            )),
            SelectionVariant::First => Ok(results
                .into_iter()
                .next()
                .map(|(k, v)| {
                    TypedValue::new(
                        ExpressionValue::Map(vec![(k, v)]),
                        TypeDescriptor::from_type_name("Map"),
                    )
                })
                .unwrap_or(TypedValue::null())),
            SelectionVariant::Last => Ok(results
                .into_iter()
                .last()
                .map(|(k, v)| {
                    TypedValue::new(
                        ExpressionValue::Map(vec![(k, v)]),
                        TypeDescriptor::from_type_name("Map"),
                    )
                })
                .unwrap_or(TypedValue::null())),
        }
    }
}

impl SpelNode for Selection {
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

        // null-safe 检查（对标 Spring Selection null-safe）
        if self.null_safe && source.is_null() {
            return Ok(TypedValue::null());
        }

        match source.value() {
            ExpressionValue::List(items) => self.select_list(items, state),
            ExpressionValue::Map(entries) => self.select_map(entries, state),
            _ => Err(EvaluationException::new(
                "",
                None,
                "选择运算需要列表或映射操作数",
            )),
        }
    }

    fn child_count(&self) -> usize {
        1
    }

    fn get_child(&self, i: usize) -> Option<&dyn SpelNode> {
        if i == 0 { Some(&*self.criteria) } else { None }
    }

    fn start_position(&self) -> usize {
        self.criteria.start_position().saturating_sub(2)
    }

    fn end_position(&self) -> usize {
        self.criteria.end_position() + 1
    }

    fn to_string_ast(&self) -> String {
        let prefix = match self.variant {
            SelectionVariant::All => "?",
            SelectionVariant::First => "^",
            SelectionVariant::Last => "$",
        };
        format!("[{}({})]", prefix, self.criteria.to_string_ast())
    }
}
