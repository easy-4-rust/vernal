//! 选择/过滤运算符节点。
//!
//! 对标 Spring `Selection`：`?[]` / `^[]` / `$[]`

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 选择变体。
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
///
/// 注：当前实现未做 push/pop root context（Phase G 完善 `ExpressionState` 后接入）。
pub struct Selection {
    criteria: Box<dyn SpelNode>,
    variant: SelectionVariant,
}

impl Selection {
    /// 创建 Selection 节点。
    ///
    /// # 参数
    ///
    /// - `criteria` — 过滤条件表达式
    /// - `variant` — 选择变体（ALL/FIRST/LAST）
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
