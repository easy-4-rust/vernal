//! 方法调用节点。
//!
//! 对标 Spring 的 `MethodReference`：`method(arg1, arg2)`

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::TypedValue;

/// 方法调用节点。
///
/// 通过 MethodResolver 解析并执行方法。
/// 对标 Spring 的 `org.springframework.expression.spel.ast.MethodReference`。
pub struct MethodReference {
    name: String,
    arguments: Vec<Box<dyn SpelNode>>,
    null_safe: bool,
}

impl MethodReference {
    #[must_use]
    pub fn new(name: String, arguments: Vec<Box<dyn SpelNode>>, null_safe: bool) -> Self {
        Self {
            name,
            arguments,
            null_safe,
        }
    }
}

impl SpelNode for MethodReference {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        // 求值所有参数
        let mut args = Vec::with_capacity(self.arguments.len());
        for arg in &self.arguments {
            args.push(arg.get_value(context)?);
        }

        // 通过 MethodResolver 解析方法
        let root = context.root_object().clone();
        for resolver in context.method_resolvers() {
            let arg_types: Vec<_> = args.iter().map(|a| a.type_descriptor().clone()).collect();
            if let Ok(Some(executor)) = resolver.resolve(context, &root, &self.name, &arg_types) {
                return executor
                    .execute(context, &root, &args)
                    .map_err(|e| EvaluationException::new(&self.name, None, e.to_string()));
            }
        }

        Err(EvaluationException::new(
            &self.name,
            None,
            format!("方法 '{}' 未找到", self.name),
        ))
    }

    fn child_count(&self) -> usize {
        self.arguments.len()
    }

    fn to_string_ast(&self) -> String {
        let args: Vec<_> = self.arguments.iter().map(|a| a.to_string_ast()).collect();
        format!("{}({})", self.name, args.join(", "))
    }
}
