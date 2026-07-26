//! 索引访问节点。
//!
//! 对标 Spring 的 `Indexer`：`[index]`

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypedValue};

/// 索引访问节点。
pub struct Indexer {
    index: Box<dyn SpelNode>,
}

impl Indexer {
    #[must_use]
    pub fn new(index: Box<dyn SpelNode>) -> Self {
        Self { index }
    }
}

impl SpelNode for Indexer {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        let index = self.index.get_value(context)?;
        let root = context.root_object();

        match (root.value(), index.value()) {
            (ExpressionValue::List(list), ExpressionValue::Int(i)) => {
                let idx = *i as usize;
                if idx < list.len() {
                    Ok(list[idx].clone())
                } else {
                    Err(EvaluationException::new(
                        "",
                        None,
                        format!("索引 {} 超出列表长度 {}", idx, list.len()),
                    ))
                }
            }
            (ExpressionValue::Map(map), key) => {
                for (k, v) in map {
                    if k.value() == key {
                        return Ok(v.clone());
                    }
                }
                Ok(TypedValue::null())
            }
            _ => Err(EvaluationException::new(
                "",
                None,
                "索引操作不支持的操作数类型",
            )),
        }
    }

    fn to_string_ast(&self) -> String {
        format!("[{}]", self.index.to_string_ast())
    }
}
