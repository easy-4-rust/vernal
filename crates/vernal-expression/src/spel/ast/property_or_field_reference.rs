//! 属性/字段引用。
//!
//! 对标 Spring 的 `PropertyOrFieldReference`：`name`、`age`、`?.name`

use super::spel_node::SpelNode;
use super::super::expression_state::ExpressionState;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::TypedValue;

/// 属性/字段引用节点。
///
/// 通过 PropertyAccessor 读取目标对象的属性。
/// 支持 null-safe 访问（`?.name`）：目标为 null 时返回 null 而非报错。
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

    #[must_use]
    pub fn is_null_safe(&self) -> bool {
        self.null_safe
    }
}

impl SpelNode for PropertyOrFieldReference {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        let mut state = ExpressionState::new(context);
        self.get_value_state(&mut state)
    }

    fn get_value_state(
        &self,
        state: &mut ExpressionState,
    ) -> Result<TypedValue, EvaluationException> {
        let root = state.active_context_object().clone();

        // null-safe 检查：目标为 null 时返回 null（对标 Spring PropertyOrFieldReference）
        if self.null_safe && root.is_null() {
            return Ok(TypedValue::null());
        }

        for accessor in state.property_accessors() {
            if accessor.can_read(state.evaluation_context(), &root, &self.name) {
                return accessor
                    .read(state.evaluation_context(), &root, &self.name)
                    .map_err(|e| EvaluationException::new(&self.name, None, e.to_string()));
            }
        }
        Err(EvaluationException::new(
            &self.name,
            None,
            format!("属性 '{}' 未找到", self.name),
        ))
    }

    fn is_null_safe(&self) -> bool {
        self.null_safe
    }

    fn is_writable(
        &self,
        _context: &dyn EvaluationContext,
    ) -> bool {
        true
    }

    fn to_string_ast(&self) -> String {
        if self.null_safe {
            format!("?.{}", self.name)
        } else {
            self.name.clone()
        }
    }
}
