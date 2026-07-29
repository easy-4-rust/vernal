//! AbstractBeanDefinitionReader — Spring 风格的 Bean 定义读取器抽象基类。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.AbstractBeanDefinitionReader`。
//!
//! 提供 Bean 定义读取器的共享实现：注册表引用、Bean 名称生成器、默认值。

use std::fmt;

use crate::bean_definition_defaults::BeanDefinitionDefaults;
use crate::bean_definition_reader::BeanDefinitionReader;
use crate::bean_definition_registry::BeanDefinitionRegistry;
use crate::bean_definition_resource::BeanDefinitionResource;
use crate::bean_name_generator::BeanNameGenerator;

/// Spring 风格的 Bean 定义读取器抽象基类。
///
/// 对应 Spring 的 `AbstractBeanDefinitionReader`。
///
/// 持有一个 `BeanDefinitionRegistry` 引用、一个可选的 `BeanNameGenerator`
/// 以及一组 `BeanDefinitionDefaults`。具体读取逻辑由子类实现。
pub struct AbstractBeanDefinitionReader {
    /// 注册表引用。
    registry: Box<dyn BeanDefinitionRegistry>,
    /// Bean 名称生成器。
    bean_name_generator: Option<Box<dyn BeanNameGenerator>>,
    /// Bean 定义默认值。
    defaults: BeanDefinitionDefaults,
}

impl fmt::Debug for AbstractBeanDefinitionReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AbstractBeanDefinitionReader")
            .field(
                "bean_name_generator",
                &self.bean_name_generator.as_ref().map(|_| "(..)"),
            )
            .field("defaults", &self.defaults)
            .finish_non_exhaustive()
    }
}

impl AbstractBeanDefinitionReader {
    /// 创建新的 AbstractBeanDefinitionReader。
    pub fn new(registry: Box<dyn BeanDefinitionRegistry>) -> Self {
        Self {
            registry,
            bean_name_generator: None,
            defaults: BeanDefinitionDefaults::new(),
        }
    }

    /// 获取注册表的引用。
    ///
    /// 对应 Spring 的 `getRegistry()`。
    pub fn get_registry(&self) -> &dyn BeanDefinitionRegistry {
        self.registry.as_ref()
    }

    /// 获取注册表的可变引用。
    pub fn get_registry_mut(&mut self) -> &mut dyn BeanDefinitionRegistry {
        self.registry.as_mut()
    }

    /// 获取 Bean 名称生成器。
    pub fn bean_name_generator(&self) -> Option<&dyn BeanNameGenerator> {
        self.bean_name_generator.as_deref()
    }

    /// 设置 Bean 名称生成器。
    pub fn set_bean_name_generator(&mut self, generator: Box<dyn BeanNameGenerator>) {
        self.bean_name_generator = Some(generator);
    }

    /// 获取 Bean 定义默认值。
    pub fn defaults(&self) -> &BeanDefinitionDefaults {
        &self.defaults
    }

    /// 获取 Bean 定义默认值的可变引用。
    pub fn defaults_mut(&mut self) -> &mut BeanDefinitionDefaults {
        &mut self.defaults
    }
}

impl BeanDefinitionReader for AbstractBeanDefinitionReader {
    fn registry(&self) -> &dyn BeanDefinitionRegistry {
        self.registry.as_ref()
    }

    fn load_bean_definitions(
        &self,
        _resource: &dyn BeanDefinitionResource,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        // 基类默认实现：具体子类应覆盖此方法
        Ok(0)
    }

    fn bean_name_generator(&self) -> Option<&dyn BeanNameGenerator> {
        self.bean_name_generator.as_deref()
    }

    fn set_bean_name_generator(&mut self, generator: Box<dyn BeanNameGenerator>) {
        self.bean_name_generator = Some(generator);
    }
}
