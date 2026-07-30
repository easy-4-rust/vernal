//! 索引访问节点。
//!
//! 对标 Spring 的 `Indexer`：`[index]`。
//!
//! 支持的索引类型（对标 Spring Indexer）：
//! - List[Int] — 列表索引
//! - Map[key] — Map 键查找
//! - String[Int] — 字符串字符索引
//! - Object[property] — 对象属性访问（通过 PropertyAccessor）

use super::spel_node::SpelNode;
use super::super::expression_state::ExpressionState;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::spel::spel_message::SpelMessage;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 索引访问节点。
pub struct Indexer {
    index: Box<dyn SpelNode>,
    null_safe: bool,
}

impl Indexer {
    #[must_use]
    pub fn new(index: Box<dyn SpelNode>) -> Self {
        Self { index, null_safe: false }
    }

    /// 创建 null-safe Indexer 节点。
    #[must_use]
    pub fn new_null_safe(index: Box<dyn SpelNode>) -> Self {
        Self { index, null_safe: true }
    }
}

impl SpelNode for Indexer {
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
        let index = self.index.get_value_state(state)?;
        let target = state.active_context_object().clone();

        // null-safe 检查（对标 Spring Indexer null-safe）
        if self.null_safe && target.is_null() {
            return Ok(TypedValue::null());
        }

        match (target.value(), index.value()) {
            // List[Int] — 列表索引（支持自动增长）
            (ExpressionValue::List(list), ExpressionValue::Int(i)) => {
                let idx = *i as usize;
                if idx < list.len() {
                    Ok(list[idx].clone())
                } else if state.auto_grow_collections() && idx < state.maximum_auto_grow_size() {
                    // 对标 Spring Indexer: autoGrowCollections 支持
                    // 当索引超出范围但小于 maximumAutoGrowSize 时，返回 null
                    // 注意：实际的列表增长需要在 setValue 时完成
                    Ok(TypedValue::null())
                } else {
                    Err(EvaluationException::new(
                        "",
                        None,
                        SpelMessage::CollectionIndexOutOfBounds
                            .format_message(&[&list.len().to_string(), &i.to_string()]),
                    ))
                }
            }

            // Map[key] — Map 键查找
            (ExpressionValue::Map(map), key) => {
                for (k, v) in map {
                    if k.value() == key {
                        return Ok(v.clone());
                    }
                }
                Ok(TypedValue::null())
            }

            // String[Int] — 字符串字符索引（对标 Spring Indexer String 处理）
            (ExpressionValue::String(s), ExpressionValue::Int(i)) => {
                let idx = *i as usize;
                if idx < s.len() {
                    let ch = s.chars().nth(idx).unwrap_or('\0');
                    Ok(TypedValue::new(
                        ExpressionValue::Char(ch),
                        TypeDescriptor::Primitive(crate::type_descriptor::PrimitiveKind::Char),
                    ))
                } else {
                    Err(EvaluationException::new(
                        "",
                        None,
                        SpelMessage::StringIndexOutOfBounds
                            .format_message(&[&s.len().to_string(), &i.to_string()]),
                    ))
                }
            }

            // 通过 IndexAccessor 访问（对标 Spring Indexer 的 IndexAccessor 集成）
            _ => {
                let index_accessors = state.evaluation_context().index_accessors();
                for accessor in &index_accessors {
                    if accessor.can_read(state.evaluation_context(), &target, &index) {
                        return accessor
                            .read(state.evaluation_context(), &target, &index)
                            .map_err(|e| {
                                EvaluationException::new(
                                    "",
                                    None,
                                    SpelMessage::ExceptionDuringIndexRead
                                        .format_message(&[&index.to_string(), &e.to_string()]),
                                )
                            });
                    }
                }

                // Object[property] — 对象属性访问（通过 PropertyAccessor）
                if let ExpressionValue::String(prop_name) = index.value() {
                    let accessors = state.property_accessors();
                    for accessor in &accessors {
                        if accessor.can_read(state.evaluation_context(), &target, prop_name) {
                            return accessor
                                .read(state.evaluation_context(), &target, prop_name)
                                .map_err(|e| {
                                    EvaluationException::new(
                                        "",
                                        None,
                                        SpelMessage::ExceptionDuringIndexRead
                                            .format_message(&[prop_name, &e.to_string()]),
                                    )
                                });
                        }
                    }
                }

                Err(EvaluationException::new(
                    "",
                    None,
                    SpelMessage::IndexingNotSupportedForType
                        .format_message(&[&target.type_descriptor().name()]),
                ))
            }
        }
    }

    fn to_string_ast(&self) -> String {
        format!("[{}]", self.index.to_string_ast())
    }
}
