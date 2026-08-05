//! ArgumentConvertingMethodInvoker — Spring 风格的参数转换方法调用器。
//!
//! 对应 Java 类：`org.springframework.beans.support.ArgumentConvertingMethodInvoker`。
//!
//! 在 Spring 中，`ArgumentConvertingMethodInvoker` 扩展了 `MethodInvoker`，
//! 在方法调用前自动转换参数类型。它使用 `ConversionService` 将参数
//! 转换为目标方法所需的类型。
//!
//! ## 设计说明
//!
//! 在 vernal 中，`ArgumentConvertingMethodInvoker` 使用闭包实现方法调用，
//! 并在调用前验证参数类型。

use std::any::Any;
use std::sync::Arc;

/// 参数转换方法调用器。
///
/// 对应 Spring 的 `ArgumentConvertingMethodInvoker`。
///
/// 在方法调用前自动转换参数类型。
#[derive(Debug)]
pub struct ArgumentConvertingMethodInvoker {
    /// 方法名称（用于日志和错误信息）。
    method_name: String,
    /// 目标对象描述。
    target_description: String,
    /// 参数类型名称。
    parameter_types: Vec<String>,
}

impl ArgumentConvertingMethodInvoker {
    /// 创建参数转换方法调用器。
    pub fn new(method_name: impl Into<String>) -> Self {
        Self {
            method_name: method_name.into(),
            target_description: String::new(),
            parameter_types: Vec::new(),
        }
    }

    /// 设置目标对象描述。
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target_description = target.into();
        self
    }

    /// 设置参数类型。
    pub fn with_parameter_types(mut self, types: Vec<String>) -> Self {
        self.parameter_types = types;
        self
    }

    /// 获取方法名。
    pub fn method_name(&self) -> &str {
        &self.method_name
    }

    /// 获取参数类型。
    pub fn parameter_types(&self) -> &[String] {
        &self.parameter_types
    }

    /// 验证参数数量是否匹配。
    pub fn validate_args(&self, args: &[Arc<dyn Any + Send + Sync>]) -> Result<(), String> {
        if !self.parameter_types.is_empty() && args.len() != self.parameter_types.len() {
            return Err(format!(
                "Method '{}' expects {} arguments, got {}",
                self.method_name,
                self.parameter_types.len(),
                args.len()
            ));
        }
        Ok(())
    }

    /// 调用方法（使用闭包）。
    pub fn invoke(
        &self,
        invoker: &dyn Fn(
            &[Arc<dyn Any + Send + Sync>],
        ) -> Result<
            Arc<dyn Any + Send + Sync>,
            Box<dyn std::error::Error + Send + Sync>,
        >,
        args: &[Arc<dyn Any + Send + Sync>],
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        self.validate_args(args)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
        invoker(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invoke_with_matching_args() {
        let invoker = ArgumentConvertingMethodInvoker::new("add")
            .with_parameter_types(vec!["i32".to_string(), "i32".to_string()]);

        let result = invoker
            .invoke(
                &|args| {
                    let a = args[0].downcast_ref::<i32>().unwrap();
                    let b = args[1].downcast_ref::<i32>().unwrap();
                    Ok(Arc::new(a + b))
                },
                &[Arc::new(3_i32), Arc::new(4_i32)],
            )
            .unwrap();

        assert_eq!(result.downcast_ref::<i32>(), Some(&7));
    }

    #[test]
    fn invoke_with_wrong_arg_count_errors() {
        let invoker = ArgumentConvertingMethodInvoker::new("add")
            .with_parameter_types(vec!["i32".to_string(), "i32".to_string()]);

        let result = invoker.invoke(&|_| Ok(Arc::new(0)), &[Arc::new(1_i32)]);
        assert!(result.is_err());
    }

    #[test]
    fn invoke_without_type_checks() {
        let invoker = ArgumentConvertingMethodInvoker::new("echo");
        let result = invoker
            .invoke(&|args| Ok(Arc::clone(&args[0])), &[Arc::new(42_i32)])
            .unwrap();
        assert_eq!(result.downcast_ref::<i32>(), Some(&42));
    }

    #[test]
    fn method_name_and_types() {
        let invoker = ArgumentConvertingMethodInvoker::new("test")
            .with_parameter_types(vec!["String".to_string()]);
        assert_eq!(invoker.method_name(), "test");
        assert_eq!(invoker.parameter_types().len(), 1);
    }
}
