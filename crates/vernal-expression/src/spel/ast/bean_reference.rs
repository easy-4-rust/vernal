//! Bean 引用节点。
//!
//! 对标 Spring 的 `BeanReference`：`@beanName`

use crate::typed_value::TypedValue;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use super::spel_node::SpelNode;

/// Bean 引用节点。
pub struct BeanReference {
    name: String,
}

impl BeanReference {
    #[must_use]
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl SpelNode for BeanReference {
    fn get_value(&self, context: &dyn EvaluationContext) -> Result<TypedValue, EvaluationException> {
        match context.bean_resolver() {
            Some(resolver) => resolver.resolve(context, &self.name)
                .map_err(|e| EvaluationException::new(&self.name, None, e.to_string())),
            None => Err(EvaluationException::new(&self.name, None, "BeanResolver 未配置")),
        }
    }

    fn to_string_ast(&self) -> String {
        format!("@{}", self.name)
    }
}
