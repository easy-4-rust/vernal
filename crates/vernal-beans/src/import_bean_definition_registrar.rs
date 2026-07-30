//! ImportBeanDefinitionRegistrar — 导入 Bean 定义注册器。
use crate::bean_definition_registry::BeanDefinitionRegistry;

/// 导入 Bean 定义注册器 trait。
pub trait ImportBeanDefinitionRegistrar: Send + Sync {
    fn register_bean_definitions(&self, registry: &mut dyn BeanDefinitionRegistry);
}
