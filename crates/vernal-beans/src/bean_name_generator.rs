//! BeanNameGenerator — Bean 名称生成器。
use crate::bean_definition::BeanDefinition;

/// Bean 名称生成器 trait。
pub trait BeanNameGenerator: Send + Sync {
    fn generate_bean_name(&self, definition: &dyn BeanDefinition) -> String;
}
