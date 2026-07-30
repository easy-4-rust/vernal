//! BeanDefinitionReader — Spring 风格 Bean 定义读取器。

use std::any::TypeId;

/// Spring 风格 Bean 定义读取器接口。
pub trait BeanDefinitionReader: Send + Sync {
    fn load_bean_definitions(&self, resource: &str) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;
    fn get_bean_class_count(&self) -> usize;
    fn get_registry_class_loader(&self) -> Option<TypeId>;
}
