//! 属性/字段引用。
//!
//! 对标 Spring 的 `PropertyOrFieldReference`：`name`、`age`

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::TypedValue;

/// 属性/字段引用节点。
///
/// 通过 PropertyAccessor 读取目标对象的属性。
/// 对标 Spring 的 `org.springframework.expression.spel.ast.PropertyOrFieldReference`。
pub struct PropertyOrFieldReference {
    name: String,
    null_safe: bool,
}

impl PropertyOrFieldReference {
    #[must_use]
    pub fn new(name: String, null_safe: bool) -> Self {
        Self { name, null_safe }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl SpelNode for PropertyOrFieldReference {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        let root = context.root_object().clone();
        for accessor in context.property_accessors() {
            if accessor.can_read(context, &root, &self.name) {
                return accessor
                    .read(context, &root, &self.name)
                    .map_err(|e| EvaluationException::new(&self.name, None, e.to_string()));
            }
        }
        Err(EvaluationException::new(
            &self.name,
            None,
            format!("属性 '{}' 未找到", self.name),
        ))
    }

    fn is_writable(&self) -> bool {
        true
    }

    fn to_string_ast(&self) -> String {
        self.name.clone()
    }
}
