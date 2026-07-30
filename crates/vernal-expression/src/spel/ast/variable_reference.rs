//! 变量引用节点。
//!
//! 对标 Spring 的 `VariableReference`：`#var`、`#root`、`#this`

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::TypedValue;

/// 变量引用节点。
///
/// 对标 Spring 的 `org.springframework.expression.spel.ast.VariableReference`。
pub struct VariableReference {
    name: String,
}

impl VariableReference {
    #[must_use]
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl SpelNode for VariableReference {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        match self.name.as_str() {
            "this" => Ok(context.root_object().clone()),
            "root" => Ok(context.root_object().clone()),
            _ => context.lookup_variable(&self.name).cloned().ok_or_else(|| {
                EvaluationException::new(&self.name, None, format!("变量 '#{}' 未找到", self.name))
            }),
        }
    }

    fn is_writable(&self, _context: &dyn EvaluationContext) -> bool {
        true
    }

    fn to_string_ast(&self) -> String {
        format!("#{}", self.name)
    }
}
