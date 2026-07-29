//! ConfigurationClassBeanDefinitionReader — 从配置类加载 Bean 定义。
//!
//! 对应 Java 类：`org.springframework.context.annotation.ConfigurationClassBeanDefinitionReader`。
//!
//! 将 `ConfigurationClassParser` 产出的 `ConfigurationClass` 列表转换为
//! 可注册的 Bean 定义描述，并写入 `BeanDefinitionRegistry`。

use std::collections::HashMap;

use crate::bean_definition_registry::BeanDefinitionRegistry;
use crate::configuration_class::ConfigurationClass;

/// 单个由 `@Bean` 方法派生的 Bean 定义描述。
///
/// 对应 Spring 的 `ConfigurationClassBeanDefinitionReader` 内部构造的
/// `RootBeanDefinition`（此处以简化结构表示）。
#[derive(Debug, Clone)]
pub struct BeanMethodDefinition {
    /// Bean 名称。
    pub bean_name: String,
    /// 工厂方法名。
    pub factory_method: String,
    /// 配置类全限定名（工厂 Bean）。
    pub factory_bean_name: String,
    /// 返回类型名。
    pub return_type_name: String,
}

impl BeanMethodDefinition {
    /// 创建新的 Bean 方法定义。
    pub fn new(
        bean_name: impl Into<String>,
        factory_method: impl Into<String>,
        factory_bean_name: impl Into<String>,
        return_type_name: impl Into<String>,
    ) -> Self {
        Self {
            bean_name: bean_name.into(),
            factory_method: factory_method.into(),
            factory_bean_name: factory_bean_name.into(),
            return_type_name: return_type_name.into(),
        }
    }
}

/// Spring 风格的配置类 Bean 定义读取器。
///
/// 对应 Spring 的 `ConfigurationClassBeanDefinitionReader`。
///
/// 遍历 `ConfigurationClass` 列表，为每个 `@Bean` 方法构造一个
/// `BeanMethodDefinition`，并可选择写入 `BeanDefinitionRegistry`。
pub struct ConfigurationClassBeanDefinitionReader {
    /// 加载期间生成的 Bean 方法定义（按 Bean 名称索引）。
    loaded: HashMap<String, BeanMethodDefinition>,
}

impl std::fmt::Debug for ConfigurationClassBeanDefinitionReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigurationClassBeanDefinitionReader")
            .field("loaded_count", &self.loaded.len())
            .finish()
    }
}

impl ConfigurationClassBeanDefinitionReader {
    /// 创建新的读取器。
    pub fn new() -> Self {
        Self {
            loaded: HashMap::new(),
        }
    }

    /// 从 `ConfigurationClass` 列表加载 Bean 定义。
    ///
    /// 对应 Spring 的 `loadBeanDefinitions(Set<ConfigurationClass>)`。
    pub fn load_bean_definitions(&mut self, configuration_classes: &[ConfigurationClass]) {
        for config in configuration_classes {
            let class_name = config.class_name();
            for method_name in config.bean_methods() {
                let return_type = config
                    .bean_method_details()
                    .iter()
                    .find(|m| &m.method_name == method_name)
                    .map(|m| m.return_type_name.as_str())
                    .unwrap_or("")
                    .to_owned();
                let bean_name = config
                    .bean_method_details()
                    .iter()
                    .find(|m| &m.method_name == method_name)
                    .map(|m| m.bean_name.as_str())
                    .unwrap_or(method_name.as_str())
                    .to_owned();

                let definition =
                    BeanMethodDefinition::new(bean_name, method_name, class_name, return_type);
                self.loaded.insert(definition.bean_name.clone(), definition);
            }
        }
    }

    /// 将已加载的 Bean 定义写入注册表（仅记录名称，实际 Bean 定义需由上层构造）。
    ///
    /// 返回成功写入的 Bean 名称列表。当前实现不写入完整的 `BeanDefinition`，
    /// 因为构造一个 `Box<dyn BeanDefinition>` 需要具体的实现类型。
    pub fn register_into(
        &mut self,
        registry: &mut dyn BeanDefinitionRegistry,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        // 实际 BeanDefinition 构造依赖上层具体实现；此处保留注册语义，
        // 通过检查 registry 的现有定义避免重复，并返回待注册名称列表。
        let mut registered = Vec::new();
        for bean_name in self.loaded.keys() {
            if !registry.contains_bean_definition(bean_name) {
                registered.push(bean_name.clone());
            }
        }
        Ok(registered)
    }

    /// 获取已加载的 Bean 方法定义。
    pub fn loaded_definitions(&self) -> &HashMap<String, BeanMethodDefinition> {
        &self.loaded
    }

    /// 获取指定 Bean 名称的定义。
    pub fn get(&self, bean_name: &str) -> Option<&BeanMethodDefinition> {
        self.loaded.get(bean_name)
    }

    /// 已加载数量。
    pub fn len(&self) -> usize {
        self.loaded.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.loaded.is_empty()
    }
}

impl Default for ConfigurationClassBeanDefinitionReader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration_class::{ConfigurationBeanMethod, ConfigurationClass};

    #[test]
    fn test_load_from_configuration_class() {
        let mut config = ConfigurationClass::new("com.example.AppConfig");
        config.add_bean_method_detail(ConfigurationBeanMethod::new("createFoo", "com.example.Foo"));

        let mut reader = ConfigurationClassBeanDefinitionReader::new();
        reader.load_bean_definitions(&[config]);

        assert_eq!(reader.len(), 1);
        let def = reader.get("createFoo").unwrap();
        assert_eq!(def.factory_method, "createFoo");
        assert_eq!(def.factory_bean_name, "com.example.AppConfig");
        assert_eq!(def.return_type_name, "com.example.Foo");
    }
}
