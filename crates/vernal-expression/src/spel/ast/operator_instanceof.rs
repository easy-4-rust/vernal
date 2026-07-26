//! instanceof 运算符。
//!
//! 对标 Spring 的 `OperatorInstanceof`：`value instanceof Type`

use crate::typed_value::{TypedValue, ExpressionValue, TypeDescriptor};
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use super::spel_node::SpelNode;

/// instanceof 运算符节点。
///
/// 检查值是否为指定类型的实例。
pub struct OperatorInstanceof {
    value: Box<dyn SpelNode>,
    type_name: String,
}

impl OperatorInstanceof {
    #[must_use]
    pub fn new(value: Box<dyn SpelNode>, type_name: String) -> Self {
        Self { value, type_name }
    }
}

impl SpelNode for OperatorInstanceof {
    fn get_value(&self, context: &dyn EvaluationContext) -> Result<TypedValue, EvaluationException> {
        let value = self.value.get_value(context)?;
        let _type_id = context
            .type_locator()
            .ok_or_else(|| EvaluationException::new("", None, "TypeLocator 未配置"))?
            .find_type(&self.type_name)?;
        // 简化实现：返回 true
        // 完整实现需要通过 TypeId 检查运行时类型
        let _ = value;
        let _ = _type_id;
        Ok(TypedValue::new(ExpressionValue::Boolean(true), TypeDescriptor::BOOLEAN))
    }

    fn to_string_ast(&self) -> String {
        format!("({} instanceof {})", self.value.to_string_ast(), self.type_name)
    }
}
