//! SimpleBeanDefinitionRegistry — Spring 风格的简易 BeanDefinition 注册表实现。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.SimpleBeanDefinitionRegistry`。
//!
//! `BeanDefinitionRegistry` 的最简实现：内部使用 `HashMap` 存储 Bean 定义，
//! 不参与 Bean 实例化，仅维护定义元数据。常用于测试或作为更复杂注册表的基类。

use std::collections::HashMap;

use crate::bean_definition::BeanDefinition;
use crate::bean_definition_registry::BeanDefinitionRegistry;

/// 简易 BeanDefinition 注册表。
///
/// 对应 Spring 的 `SimpleBeanDefinitionRegistry`。
///
/// 仅以 `HashMap<String, Box<dyn BeanDefinition>>` 维护定义，提供基本的增删查改。
/// 重复注册同名 Bean 会覆盖旧定义（语义对应 Spring 的 `allowBeanDefinitionOverriding = true`）。
pub struct SimpleBeanDefinitionRegistry {
    /// Bean 名称 → Bean 定义。
    bean_definitions: HashMap<String, Box<dyn BeanDefinition>>,
}

impl SimpleBeanDefinitionRegistry {
    /// 创建空的注册表。
    pub fn new() -> Self {
        Self {
            bean_definitions: HashMap::new(),
        }
    }
}

impl Default for SimpleBeanDefinitionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl BeanDefinitionRegistry for SimpleBeanDefinitionRegistry {
    fn register_bean_definition(
        &mut self,
        bean_name: String,
        definition: Box<dyn BeanDefinition>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if bean_name.is_empty() {
            return Err("Bean name must not be empty".into());
        }
        // 直接接管 Box 所有权；同名覆盖。
        self.bean_definitions.insert(bean_name, definition);
        Ok(())
    }

    fn remove_bean_definition(
        &mut self,
        bean_name: &str,
    ) -> Result<Box<dyn BeanDefinition>, Box<dyn std::error::Error + Send + Sync>> {
        match self.bean_definitions.remove(bean_name) {
            Some(definition) => Ok(definition),
            None => Err(format!("No bean definition named '{}' to remove", bean_name).into()),
        }
    }

    fn get_bean_definition(&self, bean_name: &str) -> Option<&dyn BeanDefinition> {
        self.bean_definitions
            .get(bean_name)
            .map(|boxed| boxed.as_ref())
    }

    fn contains_bean_definition(&self, bean_name: &str) -> bool {
        self.bean_definitions.contains_key(bean_name)
    }

    fn bean_definition_count(&self) -> usize {
        self.bean_definitions.len()
    }

    fn bean_definition_names(&self) -> Vec<String> {
        self.bean_definitions.keys().cloned().collect()
    }
}

impl std::fmt::Debug for SimpleBeanDefinitionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleBeanDefinitionRegistry")
            .field("bean_definition_count", &self.bean_definitions.len())
            .field(
                "bean_names",
                &self.bean_definitions.keys().collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl SimpleBeanDefinitionRegistry {
    /// 清空所有 Bean 定义。
    pub fn clear(&mut self) {
        self.bean_definitions.clear();
    }
}
