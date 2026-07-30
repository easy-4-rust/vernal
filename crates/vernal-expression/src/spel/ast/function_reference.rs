//! 函数引用节点。
//!
//! 对标 Spring 的 `FunctionReference`：`#function(args)`

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::TypedValue;

/// 函数引用节点。
pub struct FunctionReference {
    name: String,
    arguments: Vec<Box<dyn SpelNode>>,
}

impl FunctionReference {
    #[must_use]
    pub fn new(name: String, arguments: Vec<Box<dyn SpelNode>>) -> Self {
        Self { name, arguments }
    }
}

impl SpelNode for FunctionReference {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        // 查找注册的函数（通过变量查找）
        context.lookup_variable(&self.name).ok_or_else(|| {
            EvaluationException::new(&self.name, None, format!("函数 '#{}' 未找到", self.name))
        })
    }

    fn to_string_ast(&self) -> String {
        let args: Vec<_> = self.arguments.iter().map(|a| a.to_string_ast()).collect();
        format!("#{}({})", self.name, args.join(", "))
    }
}
