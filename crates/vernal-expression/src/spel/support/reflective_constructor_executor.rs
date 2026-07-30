//! 反射构造器执行器（对标 Spring `ReflectiveConstructorExecutor`）。
//!
//! 对标 Java `org.springframework.expression.spel.support.ReflectiveConstructorExecutor`。
//! 通过闭包实现构造器执行，支持参数类型匹配。
//!
//! # Spring 行为
//!
//! - 持有构造器引用（闭包）
//! - 支持参数类型匹配
//! - 支持返回值类型转换

use crate::access_exception::AccessException;
use crate::constructor_executor::ConstructorExecutor;
use crate::evaluation_context::EvaluationContext;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 反射构造器执行器（对标 Spring `ReflectiveConstructorExecutor`）。
///
/// 通过闭包实现构造器执行，支持参数类型匹配。
pub struct ReflectiveConstructorExecutor {
    /// 类型名称。
    type_name: String,
    /// 构造器闭包（接收参数，返回新实例）。
    constructor: Box<dyn Fn(&[TypedValue]) -> Result<TypedValue, AccessException> + Send + Sync>,
}

impl ReflectiveConstructorExecutor {
    /// 创建反射构造器执行器。
    pub fn new<F>(type_name: impl Into<String>, constructor: F) -> Self
    where
        F: Fn(&[TypedValue]) -> Result<TypedValue, AccessException> + Send + Sync + 'static,
    {
        Self {
            type_name: type_name.into(),
            constructor: Box::new(constructor),
        }
    }

    /// 获取类型名称。
    #[must_use]
    pub fn type_name(&self) -> &str {
        &self.type_name
    }
}

impl ConstructorExecutor for ReflectiveConstructorExecutor {
    fn execute(
        &self,
        _context: &dyn EvaluationContext,
        arguments: &[TypedValue],
    ) -> Result<TypedValue, AccessException> {
        // 调用构造器闭包
        (self.constructor)(arguments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_executor() {
        let executor = ReflectiveConstructorExecutor::new("String", |_args| {
            Ok(TypedValue::new(
                ExpressionValue::String("test".to_string()),
                TypeDescriptor::STRING,
            ))
        });
        assert_eq!(executor.type_name(), "String");
    }

    #[test]
    fn execute_constructor() {
        let executor = ReflectiveConstructorExecutor::new("String", |_args| {
            Ok(TypedValue::new(
                ExpressionValue::String("test".to_string()),
                TypeDescriptor::STRING,
            ))
        });
        let ctx = crate::spel::support::standard_evaluation_context::StandardEvaluationContext::new(
            TypedValue::null(),
        );
        let result = executor.execute(&ctx, &[]);
        assert!(result.is_ok());
        assert_eq!(
            *result.unwrap().value(),
            ExpressionValue::String("test".to_string())
        );
    }
}
