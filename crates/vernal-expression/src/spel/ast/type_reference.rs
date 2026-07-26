//! 类型引用节点。
//!
//! 对标 Spring 的 `TypeReference`：`T(String)`

use crate::typed_value::TypedValue;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use super::spel_node::SpelNode;

/// 类型引用节点。
pub struct TypeReference {
    type_name: String,
}

impl TypeReference {
    #[must_use]
    pub fn new(type_name: String) -> Self {
        Self { type_name }
    }
}

impl SpelNode for TypeReference {
    fn get_value(&self, context: &dyn EvaluationContext) -> Result<TypedValue, EvaluationException> {
        match context.type_locator() {
            Some(locator) => {
                let _type_id = locator.find_type(&self.type_name)?;
                // 返回类型信息（简化实现）
                Ok(TypedValue::null())
            }
            None => Err(EvaluationException::new(&self.type_name, None, "TypeLocator 未配置")),
        }
    }

    fn to_string_ast(&self) -> String {
        format!("T({})", self.type_name)
    }
}
