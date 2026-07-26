//! 选择/过滤运算符节点。
//!
//! 对标 Spring 的 `Selection`：`?[criteria]`、`^[first]`、`$[last]`

use crate::typed_value::{TypedValue, ExpressionValue};
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use super::spel_node::SpelNode;

/// 选择变体。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    #[must_use]
    pub fn new(criteria: Box<dyn SpelNode>, variant: SelectionVariant) -> Self {
        Self { criteria, variant }
    }
}

impl SpelNode for Selection {
    fn get_value(&self, context: &dyn EvaluationContext) -> Result<TypedValue, EvaluationException> {
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
                    SelectionVariant::All => Ok(TypedValue::new(ExpressionValue::List(results), crate::typed_value::TypeDescriptor::OBJECT)),
                    SelectionVariant::First => Ok(results.into_iter().next().unwrap_or(TypedValue::null())),
                    SelectionVariant::Last => Ok(results.into_iter().last().unwrap_or(TypedValue::null())),
                }
            }
            _ => Err(EvaluationException::new("", None, "选择运算需要列表操作数")),
        }
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
