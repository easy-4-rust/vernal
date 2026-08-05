//! SimpleBeanDefinitionRegistry — Spring 风格的简单 Bean 定义注册表。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.SimpleBeanDefinitionRegistry`。
//!
//! 在 Spring 中，`SimpleBeanDefinitionRegistry` 是 `BeanDefinitionRegistry`
//! 的简单实现，用于测试和工具类场景。它不涉及完整的 Bean 工厂功能，
//! 仅提供基本的 Bean 定义注册和查询。

use crate::factory::config::bean_definition::BeanDefinition;
use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 简单 Bean 定义注册表。
///
/// 对应 Spring 的 `SimpleBeanDefinitionRegistry`。
///
/// 提供基本的 Bean 定义注册和查询功能，
/// 适用于测试和工具类场景。
#[derive(Debug, Default)]
pub struct SimpleBeanDefinitionRegistry {
    definitions: Mutex<HashMap<String, Arc<dyn BeanDefinition>>>,
}

impl SimpleBeanDefinitionRegistry {
    /// 创建空的注册表。
    pub fn new() -> Self {
        Self::default()
    }

    /// 清空所有定义。
    pub fn clear(&mut self) {
        self.definitions.lock().unwrap().clear();
    }

    /// 获取已注册的定义数量。
    pub fn size(&self) -> usize {
        self.definitions.lock().unwrap().len()
    }

    /// 是否包含指定名称的定义。
    pub fn contains(&self, name: &str) -> bool {
        self.definitions.lock().unwrap().contains_key(name)
    }

    /// 获取所有已注册的名称。
    pub fn names(&self) -> Vec<String> {
        self.definitions.lock().unwrap().keys().cloned().collect()
    }
}

impl BeanDefinitionRegistry for SimpleBeanDefinitionRegistry {
    fn register_bean_definition(
        &mut self,
        name: String,
        def: Box<dyn BeanDefinition>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.definitions
            .lock()
            .unwrap()
            .insert(name, Arc::from(def));
        Ok(())
    }

    fn remove_bean_definition(
        &mut self,
        name: &str,
    ) -> Result<Box<dyn BeanDefinition>, Box<dyn std::error::Error + Send + Sync>> {
        self.definitions
            .lock()
            .unwrap()
            .remove(name)
            .map(|_arc| {
                // 将 Arc 转换为 Box - 创建一个空壳
                Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new())
                    as Box<dyn BeanDefinition>
            })
            .ok_or_else(|| format!("No bean definition with name '{}'", name).into())
    }

    fn get_bean_definition(&self, _name: &str) -> Option<&'static dyn BeanDefinition> {
        // Note: 由于 Mutex 生命周期限制，无法返回引用
        None
    }

    fn contains_bean_definition(&self, name: &str) -> bool {
        self.definitions.lock().unwrap().contains_key(name)
    }

    fn bean_definition_count(&self) -> usize {
        self.definitions.lock().unwrap().len()
    }

    fn bean_definition_names(&self) -> Vec<String> {
        self.definitions.lock().unwrap().keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::support::root_bean_definition::RootBeanDefinition;

    #[test]
    fn new_registry_is_empty() {
        let registry = SimpleBeanDefinitionRegistry::new();
        assert_eq!(registry.size(), 0);
        assert!(registry.names().is_empty());
    }

    #[test]
    fn register_and_check() {
        let mut registry = SimpleBeanDefinitionRegistry::new();
        let def = RootBeanDefinition::new();
        registry
            .register_bean_definition("myBean".to_string(), Box::new(def))
            .unwrap();

        assert!(registry.contains("myBean"));
        assert!(registry.contains_bean_definition("myBean"));
        assert_eq!(registry.size(), 1);
    }

    #[test]
    fn remove_bean_definition() {
        let mut registry = SimpleBeanDefinitionRegistry::new();
        let def = RootBeanDefinition::new();
        registry
            .register_bean_definition("test".to_string(), Box::new(def))
            .unwrap();

        assert!(registry.contains("test"));
        registry.remove_bean_definition("test").unwrap();
        assert!(!registry.contains("test"));
    }

    #[test]
    fn remove_nonexistent_fails() {
        let mut registry = SimpleBeanDefinitionRegistry::new();
        let result = registry.remove_bean_definition("missing");
        assert!(result.is_err());
    }

    #[test]
    fn bean_definition_names() {
        let mut registry = SimpleBeanDefinitionRegistry::new();
        registry
            .register_bean_definition("a".to_string(), Box::new(RootBeanDefinition::new()))
            .unwrap();
        registry
            .register_bean_definition("b".to_string(), Box::new(RootBeanDefinition::new()))
            .unwrap();

        let mut names = registry.bean_definition_names();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }
}
