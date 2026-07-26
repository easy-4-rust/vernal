//! 整数字面量。
//!
//! 对标 Spring 的 `IntLiteral`：`42`

use crate::typed_value::{TypedValue, ExpressionValue, TypeDescriptor};
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use super::spel_node::SpelNode;
use super::literal::LiteralNode;

/// 整数字面量节点。
///
/// 对标 Spring 的 `org.springframework.expression.spel.ast.IntLiteral`。
#[derive(Debug, Clone)]
pub struct IntLiteral {
    /// 字面量值
    value: i64,
    /// 原始字符串
    original: String,
}

impl IntLiteral {
    /// 创建整数字面量。
    #[must_use]
    pub fn new(value: i64, original: String) -> Self {
        Self { value, original }
    }

    /// 获取整数值。
    #[must_use]
    pub fn value(&self) -> i64 {
        self.value
    }
}

impl SpelNode for IntLiteral {
    fn get_value(&self, _context: &dyn EvaluationContext) -> Result<TypedValue, EvaluationException> {
        Ok(TypedValue::new(ExpressionValue::Int(self.value), TypeDescriptor::INT))
    }

    fn to_string_ast(&self) -> String {
        self.original.clone()
    }
}

impl LiteralNode for IntLiteral {
    fn literal_value(&self) -> &TypedValue {
        // 静态值在 get_value 中创建
        unreachable!("IntLiteral 使用 get_value 直接返回")
    }
}
