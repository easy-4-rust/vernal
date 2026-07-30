//! 类型引用节点。
//!
//! 对标 Spring 的 `TypeReference`：`T(String)`。
//!
//! `T(type)` 表达式返回指定类型的 `TypeId`，用于 instanceof 检查
//! 和类型相关操作。

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 类型引用节点。
///
/// 对标 Spring `TypeReference`：`T(String)` 返回类型对象。
pub struct TypeReference {
    type_name: String,
}

impl TypeReference {
    #[must_use]
    pub fn new(type_name: String) -> Self {
        Self { type_name }
    }

    /// 获取类型名。
    #[must_use]
    pub fn type_name(&self) -> &str {
        &self.type_name
    }
}

impl SpelNode for TypeReference {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        match context.type_locator() {
            Some(locator) => {
                let type_id = locator.find_type(&self.type_name)?;
                // 对标 Spring: T(String) 返回 Class<String> 对象
                // Rust 中返回 TypeId 作为 Object 值
                Ok(TypedValue::new(
                    ExpressionValue::object(type_id),
                    TypeDescriptor::from_type_name(&self.type_name),
                ))
            }
            None => Err(EvaluationException::new(
                &self.type_name,
                None,
                "TypeLocator 未配置",
            )),
        }
    }

    fn to_string_ast(&self) -> String {
        format!("T({})", self.type_name)
    }
}
