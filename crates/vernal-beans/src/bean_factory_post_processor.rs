//! BeanFactoryPostProcessor — BeanFactory 后处理器 trait。
use crate::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
use std::fmt;

/// BeanFactory 后处理器 trait。
pub trait BeanFactoryPostProcessor: Send + Sync + fmt::Debug {
    fn post_process_bean_factory(&self, bean_factory: &mut dyn ConfigurableListableBeanFactory) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
