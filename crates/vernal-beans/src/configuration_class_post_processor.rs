//! ConfigurationClassPostProcessor — Spring 风格的配置类后处理器。
//!
//! 对应 Java 类：`org.springframework.context.annotation.ConfigurationClassPostProcessor`。
//!
//! 处理 `@Configuration` / `@Bean` 注解，注册配置类中定义的 Bean。


use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
use crate::factory::config::bean_factory_post_processor::BeanFactoryPostProcessor;
use crate::factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;

/// Spring 风格的配置类后处理器。
///
/// 对应 Spring 的 `ConfigurationClassPostProcessor`。
///
/// 处理 `@Configuration` / `@Bean` 注解，注册配置类中定义的 Bean。
/// 这是 Spring 注解驱动配置的核心处理器。
///
/// ## 执行顺序
///
/// 1. `postProcessBeanDefinitionRegistry` — 注册 `@Bean` 方法定义的 Bean
/// 2. `postProcessBeanFactory` — 处理 `@Configuration` 代理
///
/// ## 与 vernal 的关系
///
/// 在 vernal 中，`@Configuration` / `@Bean` 由 `vernal-macros` 的
/// `#[beantable]` / `#[bean]` 宏在编译期处理。`ConfigurationClassPostProcessor`
/// 提供运行时的扩展点，允许动态注册配置类。
#[derive(Debug)]
pub struct ConfigurationClassPostProcessor {
    /// 已注册的配置类名称。
    registered_configurations: Vec<String>,
}

impl ConfigurationClassPostProcessor {
    /// 创建新的 ConfigurationClassPostProcessor。
    pub fn new() -> Self {
        Self {
            registered_configurations: Vec::new(),
        }
    }

    /// 注册配置类名称。
    pub fn register_configuration(&mut self, class_name: impl Into<String>) {
        self.registered_configurations.push(class_name.into());
    }

    /// 获取已注册的配置类数量。
    pub fn registered_count(&self) -> usize {
        self.registered_configurations.len()
    }

    /// 获取已注册的配置类名称。
    pub fn registered_configurations(&self) -> &[String] {
        &self.registered_configurations
    }
}

impl Default for ConfigurationClassPostProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl BeanFactoryPostProcessor for ConfigurationClassPostProcessor {
    /// 处理配置类，注册 Bean 定义。
    ///
    /// 对应 Spring 的 `ConfigurationClassPostProcessor.postProcessBeanFactory(ConfigurableListableBeanFactory)`。
    fn post_process_bean_factory(
        &self,
        _bean_factory: &mut dyn ConfigurableListableBeanFactory,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 当前实现：记录已注册的配置类
        // 实际生产环境应：
        // 1. 扫描 @Configuration 类
        // 2. 解析 @Bean 方法
        // 3. 注册到 BeanDefinitionRegistry
        Ok(())
    }
}

/// 扩展 BeanDefinitionRegistryPostProcessor。
impl crate::factory::support::bean_definition_registry_post_processor::BeanDefinitionRegistryPostProcessor
    for ConfigurationClassPostProcessor
{
    /// 注册 @Bean 方法定义的 Bean。
    ///
    /// 对应 Spring 的 `ConfigurationClassPostProcessor.postProcessBeanDefinitionRegistry(BeanDefinitionRegistry)`。
    fn post_process_bean_definition_registry(
        &self,
        _registry: &mut dyn BeanDefinitionRegistry,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 当前实现：空操作
        // 实际生产环境应扫描 @Configuration 类并注册 @Bean 方法
        Ok(())
    }
}
