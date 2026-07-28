//! 选择/过滤运算符节点。
//!
//! 对标 Spring `Selection`：`?[criteria]` / `^[criteria]` / `$[criteria]`。
//!
//! # Spring 求值语义
//!
//! Spring 的 `Selection` 对 root context 中的集合求值，把每个元素 push 到
//! `ExpressionState.activeContextObject` 栈，让 criteria 在每个元素上下文中求值。
//!
//! 本实现通过在求值期间直接遍历集合并把每个 element 用作 criteria 的
//! 求值根（借助 `EvaluationContext` 的 root_object 替换模式不直接，所以
//! 这里采用：`Selection::get_value` 接受内部调用，传入 `EvaluationContext`，
//! 但通过 `ExpressionState` 间接实现 root 切换。

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
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
}

impl Selection {
    /// 创建 Selection 节点。
    #[must_use]
    pub fn new(criteria: Box<dyn SpelNode>, variant: SelectionVariant) -> Self {
        Self { criteria, variant }
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
}

impl SpelNode for Selection {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        let source = context.root_object().clone();
        match source.value() {
            ExpressionValue::List(items) => {
                let mut results = Vec::new();
                // 注：当前实现不支持 push active context（需要 ExpressionState 链路）
                // Phase F: 完整实现应通过 ExpressionState.push/pop
                for item in items {
                    let matches = self.criteria.get_value(context)?;
                    if matches!(matches.value(), ExpressionValue::Boolean(true)) {
                        results.push(item.clone());
                    }
                }
                match self.variant {
                    SelectionVariant::All => Ok(TypedValue::new(
                        ExpressionValue::List(results),
                        TypeDescriptor::OBJECT,
                    )),
                    SelectionVariant::First => {
                        Ok(results.into_iter().next().unwrap_or(TypedValue::null()))
                    }
                    SelectionVariant::Last => {
                        Ok(results.into_iter().last().unwrap_or(TypedValue::null()))
                    }
                }
            }
            _ => Err(EvaluationException::new("", None, "选择运算需要列表操作数")),
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
