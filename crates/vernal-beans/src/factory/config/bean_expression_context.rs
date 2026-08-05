//! BeanExpressionContext — 对应 Spring `org.springframework.beans.factory.config.BeanExpressionContext`。
//!
//! Bean 表达式上下文。

use std::collections::HashMap;

/// Bean 表达式上下文。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.BeanExpressionContext`。
///
/// 提供了 Bean 表达式求值所需的上下文信息。
#[derive(Debug)]
pub struct BeanExpressionContext {
    /// 变量存储。
    variables: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl BeanExpressionContext {
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// 设置变量。
    pub fn set_variable(
        &mut self,
        name: impl Into<String>,
        value: impl std::any::Any + Send + Sync,
    ) {
        self.variables.insert(name.into(), Box::new(value));
    }

    /// 获取变量。
    pub fn get_variable(&self, name: &str) -> Option<&dyn std::any::Any> {
        self.variables
            .get(name)
            .map(|v| v.as_ref() as &dyn std::any::Any)
    }

    /// 判断是否变量。
    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }
}

impl Default for BeanExpressionContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context() {
        let mut ctx = BeanExpressionContext::new();
        assert!(!ctx.has_variable("key"));

        ctx.set_variable("key", String::from("value"));
        assert!(ctx.has_variable("key"));
    }
}
