//! BeanDefinitionCustomizer — Bean 定义定制器。
use std::any::Any;
use std::fmt;

/// Bean 定义定制器 trait。
pub trait BeanDefinitionCustomizer: Send + Sync + fmt::Debug {
    fn customize(&self, bean: &mut dyn Any);
}
