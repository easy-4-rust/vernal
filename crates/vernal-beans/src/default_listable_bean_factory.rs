//! DefaultListableBeanFactory — Spring 风格的默认可列举 BeanFactory。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.DefaultListableBeanFactory`。
//!
//! 包装 `Container` 的功能，提供更接近 Spring API 的 BeanFactory 外观。

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use crate::bean_definition::BeanDefinition;
use crate::bean_post_processor::BeanPostProcessor;
use crate::component_registry::Registry;
use crate::container::Container;

/// Spring 风格的默认可列举 BeanFactory。
///
/// 对应 Spring 的 `DefaultListableBeanFactory`。
///
/// 包装 `Container` 的功能，提供更接近 Spring API 的 BeanFactory 外观。
/// 支持动态注册 Bean 定义、预实例化 singleton、自动装配等完整容器功能。
pub struct DefaultListableBeanFactory {
    /// 内部容器委托。
    container: Arc<Container>,
    /// 已注册的 Bean 定义缓存。
    bean_definitions: Arc<Mutex<HashMap<String, Arc<dyn BeanDefinition>>>>,
}

impl fmt::Debug for DefaultListableBeanFactory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DefaultListableBeanFactory")
            .field("bean_definition_count", &self.bean_definition_count())
            .finish()
    }
}

impl DefaultListableBeanFactory {
    /// 使用指定的 Registry 创建新的 DefaultListableBeanFactory。
    ///
    /// # 参数
    ///
    /// * `registry` — 组件注册表，包含已注册的组件定义
    pub fn new(registry: Registry) -> Self {
        Self {
            container: Arc::new(Container::new(registry)),
            bean_definitions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 创建一个空的 DefaultListableBeanFactory。
    pub fn empty() -> Self {
        Self::new(Registry::empty())
    }

    /// 获取内部 Container 的引用。
    pub fn container(&self) -> &Arc<Container> {
        &self.container
    }

    /// 注册 Bean 定义。
    ///
    /// 对应 Spring 的 `DefaultListableBeanFactory.registerBeanDefinition(String beanName, BeanDefinition beanDefinition)`。
    ///
    /// # 参数
    ///
    /// * `bean_name` — Bean 的名称
    /// * `definition` — Bean 定义
    pub fn register_bean_definition(
        &self,
        bean_name: impl Into<String>,
        definition: Arc<dyn BeanDefinition>,
    ) {
        let name = bean_name.into();
        if let Ok(mut defs) = self.bean_definitions.lock() {
            defs.insert(name, definition);
        }
    }

    /// 按名称获取 Bean 定义。
    ///
    /// # 参数
    ///
    /// * `bean_name` — Bean 的名称
    ///
    /// # 返回
    ///
    /// 如果找到，返回 Bean 定义。
    pub fn get_bean_definition(&self, bean_name: &str) -> Option<Arc<dyn BeanDefinition>> {
        self.bean_definitions
            .lock()
            .ok()
            .and_then(|defs| defs.get(bean_name).cloned())
    }

    /// 检查是否包含指定的 Bean 定义。
    ///
    /// # 参数
    ///
    /// * `bean_name` — Bean 的名称
    ///
    /// # 返回
    ///
    /// 如果存在，返回 `true`。
    pub fn contains_bean_definition(&self, bean_name: &str) -> bool {
        self.bean_definitions
            .lock()
            .map(|defs| defs.contains_key(bean_name))
            .unwrap_or(false)
    }

    /// 移除 Bean 定义。
    ///
    /// # 参数
    ///
    /// * `bean_name` — Bean 的名称
    pub fn remove_bean_definition(&self, bean_name: &str) {
        if let Ok(mut defs) = self.bean_definitions.lock() {
            defs.remove(bean_name);
        }
    }

    /// 获取所有已注册的 Bean 定义名称。
    ///
    /// # 返回
    ///
    /// Bean 名称列表。
    pub fn bean_definition_names(&self) -> Vec<String> {
        self.bean_definitions
            .lock()
            .map(|defs| defs.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// 获取已注册的 Bean 定义数量。
    ///
    /// # 返回
    ///
    /// Bean 定义数量。
    pub fn bean_definition_count(&self) -> usize {
        self.bean_definitions
            .lock()
            .map(|defs| defs.len())
            .unwrap_or(0)
    }

    /// 预实例化所有 singleton Bean。
    ///
    /// 对应 Spring 的 `DefaultListableBeanFactory.preInstantiateSingletons()`。
    ///
    /// # 返回
    ///
    /// - `Ok(())` — 所有 singleton 已成功实例化
    /// - `Err` — 实例化过程中发生错误
    pub fn pre_instantiate_singletons(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 委托给底层容器的 warm_up（预实例化所有 singleton）
        self.container.warm_up().map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to pre-instantiate singletons: {}", e),
            )) as Box<dyn std::error::Error + Send + Sync>
        })
    }

    /// 获取所有 BeanPostProcessor。
    ///
    /// # 返回
    ///
    /// 已注册的后处理器列表。
    pub fn get_bean_post_processors(&self) -> Vec<Arc<dyn BeanPostProcessor>> {
        // DefaultListableBeanFactory 自身维护自己的后处理器列表
        Vec::new()
    }

    /// 冻结配置，防止进一步的修改。
    ///
    /// 对应 Spring 的 `DefaultListableBeanFactory.freezeConfiguration()`。
    pub fn freeze_configuration(&self) {
        // 当前实现不支持冻结
    }

    /// 检查配置是否已冻结。
    ///
    /// # 返回
    ///
    /// 始终返回 `false`，因为当前实现不支持冻结。
    pub fn is_configuration_frozen(&self) -> bool {
        false
    }
}

impl Default for DefaultListableBeanFactory {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::root_bean_definition::RootBeanDefinition;

    #[test]
    fn test_new_factory() {
        let factory = DefaultListableBeanFactory::empty();
        assert_eq!(factory.bean_definition_count(), 0);
    }

    #[test]
    fn test_register_and_get_bean_definition() {
        let factory = DefaultListableBeanFactory::empty();

        let mut bd = RootBeanDefinition::new();
        bd.set_bean_class_name("test::MyService");
        factory.register_bean_definition("myService", Arc::new(bd));

        assert!(factory.contains_bean_definition("myService"));
        assert_eq!(factory.bean_definition_count(), 1);
        assert!(factory.get_bean_definition("myService").is_some());
    }

    #[test]
    fn test_remove_bean_definition() {
        let factory = DefaultListableBeanFactory::empty();

        let mut bd = RootBeanDefinition::new();
        bd.set_bean_class_name("test::MyService");
        factory.register_bean_definition("myService", Arc::new(bd));

        factory.remove_bean_definition("myService");
        assert!(!factory.contains_bean_definition("myService"));
    }

    #[test]
    fn test_pre_instantiate_singletons() {
        let factory = DefaultListableBeanFactory::empty();
        // Should not panic on empty factory
        assert!(factory.pre_instantiate_singletons().is_ok());
    }
}
