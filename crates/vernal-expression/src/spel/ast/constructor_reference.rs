//! 构造器调用节点。
//!
//! 对标 Spring 的 `ConstructorReference`：`new Foo()`

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::TypedValue;

/// 构造器调用节点。
pub struct ConstructorReference {
    type_name: String,
    arguments: Vec<Box<dyn SpelNode>>,
}

impl ConstructorReference {
    /// 创建构造器调用节点。
    #[must_use]
    pub fn new(type_name: String, arguments: Vec<Box<dyn SpelNode>>) -> Self {
        Self {
            type_name,
            arguments,
        }
    }
}

impl SpelNode for ConstructorReference {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        // 通过 ConstructorResolver 解析
        let arg_types: Vec<_> = self
            .arguments
            .iter()
            .map(|_| crate::typed_value::TypeDescriptor::OBJECT)
            .collect();

        for resolver in context.constructor_resolvers() {
            if let Ok(Some(executor)) = resolver.resolve(context, &self.type_name, &arg_types) {
                let args: Vec<_> = self
                    .arguments
                    .iter()
                    .map(|a| a.get_value(context))
                    .collect::<Result<Vec<_>, _>>()?;
                return executor
                    .execute(context, &args)
                    .map_err(|e| EvaluationException::new(&self.type_name, None, e.to_string()));
            }
        }

        Err(EvaluationException::new(
            &self.type_name,
            None,
            format!("构造器 'new {}' 未找到", self.type_name),
        ))
    }

    fn to_string_ast(&self) -> String {
        let args: Vec<_> = self.arguments.iter().map(|a| a.to_string_ast()).collect();
        format!("new {}({})", self.type_name, args.join(", "))
    }
}
