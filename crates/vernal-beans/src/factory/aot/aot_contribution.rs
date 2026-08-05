//! AotContribution — 对应 Spring `org.springframework.beans.factory.aot.AotContribution`。
//!
//! AOT 贡献基类 trait。
//!
//! 在 Spring 6+ 的 AOT 处理流程中，`AotContribution` 是所有 AOT 贡献的基础接口。
//! 它定义了 AOT 贡献的基本契约：提供生成代码的类名和方法名。
//!
//! # Spring 对标
//!
//! 对应 Java 接口：`org.springframework.beans.factory.aot.AotContribution`。
//!
//! # 设计说明
//!
//! 在 Rust 中，此 trait 作为所有 AOT 贡献类型的公共基础，
//! 提供代码生成所需的元数据和执行贡献的入口点。

use std::collections::HashMap;
use std::sync::Mutex;

/// AOT 贡献 trait。
///
/// 对应 Java 接口：`org.springframework.beans.factory.aot.AotContribution`。
///
/// 所有 AOT 贡献的基础接口，定义了 AOT 代码生成的基本契约。
pub trait AotContribution: Send + Sync {
    /// 获取生成代码的目标类名。
    ///
    /// 对应 Java 方法：`String getClassName()`
    fn class_name(&self) -> &str;

    /// 获取生成代码的目标方法名。
    ///
    /// 对应 Java 方法：`String getMethodName()`
    fn method_name(&self) -> &str;

    /// 获取贡献的描述信息。
    fn description(&self) -> &str {
        "AOT contribution"
    }
}

/// AOT 贡献容器。
///
/// 用于管理和存储多个 AOT 贡献实例。
pub struct AotContributionRegistry {
    contributions: Mutex<HashMap<String, Box<dyn AotContribution>>>,
}

impl AotContributionRegistry {
    /// 创建一个新的 AotContributionRegistry。
    pub fn new() -> Self {
        Self {
            contributions: Mutex::new(HashMap::new()),
        }
    }

    /// 注册一个 AOT 贡献。
    ///
    /// # Arguments
    ///
    /// * `key` - 贡献的唯一标识
    /// * `contribution` - AOT 贡献实例
    pub fn register(&self, key: impl Into<String>, contribution: Box<dyn AotContribution>) {
        let mut contributions = self.contributions.lock().unwrap();
        contributions.insert(key.into(), contribution);
    }

    /// 获取已注册的贡献数量。
    pub fn count(&self) -> usize {
        self.contributions.lock().unwrap().len()
    }

    /// 判断是否包含指定 key 的贡献。
    pub fn contains(&self, key: &str) -> bool {
        self.contributions.lock().unwrap().contains_key(key)
    }

    /// 获取所有已注册的贡献 key。
    pub fn keys(&self) -> Vec<String> {
        self.contributions.lock().unwrap().keys().cloned().collect()
    }

    /// 清空所有已注册的贡献。
    pub fn clear(&self) {
        self.contributions.lock().unwrap().clear();
    }
}

impl Default for AotContributionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 简单的 AOT 贡献实现。
///
/// 用于测试和基本场景。
#[derive(Debug, Clone)]
pub struct SimpleAotContribution {
    class_name: String,
    method_name: String,
    description: String,
}

impl SimpleAotContribution {
    /// 创建一个新的 SimpleAotContribution。
    ///
    /// # Arguments
    ///
    /// * `class_name` - 目标类名
    /// * `method_name` - 目标方法名
    pub fn new(class_name: impl Into<String>, method_name: impl Into<String>) -> Self {
        Self {
            class_name: class_name.into(),
            method_name: method_name.into(),
            description: String::new(),
        }
    }

    /// 设置描述信息。
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
}

impl AotContribution for SimpleAotContribution {
    fn class_name(&self) -> &str {
        &self.class_name
    }

    fn method_name(&self) -> &str {
        &self.method_name
    }

    fn description(&self) -> &str {
        &self.description
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_aot_contribution() {
        let contribution = SimpleAotContribution::new("com.example.MyBean", "init");
        assert_eq!(contribution.class_name(), "com.example.MyBean");
        assert_eq!(contribution.method_name(), "init");
        assert_eq!(contribution.description(), "");
    }

    #[test]
    fn test_simple_aot_contribution_with_description() {
        let contribution = SimpleAotContribution::new("MyBean", "register")
            .with_description("Register MyBean in AOT context");
        assert_eq!(contribution.description(), "Register MyBean in AOT context");
    }

    #[test]
    fn test_registry_new() {
        let registry = AotContributionRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let registry = AotContributionRegistry::new();
        let contribution = SimpleAotContribution::new("Bean1", "method1");
        registry.register("key1", Box::new(contribution));
        assert_eq!(registry.count(), 1);
        assert!(registry.contains("key1"));
    }

    #[test]
    fn test_registry_keys() {
        let registry = AotContributionRegistry::new();
        registry.register("alpha", Box::new(SimpleAotContribution::new("A", "a")));
        registry.register("beta", Box::new(SimpleAotContribution::new("B", "b")));

        let mut keys = registry.keys();
        keys.sort();
        assert_eq!(keys, vec!["alpha", "beta"]);
    }

    #[test]
    fn test_registry_clear() {
        let registry = AotContributionRegistry::new();
        registry.register("key", Box::new(SimpleAotContribution::new("C", "c")));
        assert_eq!(registry.count(), 1);

        registry.clear();
        assert_eq!(registry.count(), 0);
        assert!(!registry.contains("key"));
    }

    #[test]
    fn test_registry_default() {
        let registry = AotContributionRegistry::default();
        assert_eq!(registry.count(), 0);
    }
}
