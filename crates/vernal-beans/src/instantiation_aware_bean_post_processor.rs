//! InstantiationAwareBeanPostProcessor — 实例化感知后处理器。
use crate::bean_post_processor::BeanPostProcessor;
use std::any::Any;
use std::sync::Arc;

/// 实例化感知后处理器 trait。
pub trait InstantiationAwareBeanPostProcessor: BeanPostProcessor {
    fn post_process_before_instantiation(&self, bean_class: &str, bean_name: &str) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }
    fn post_process_after_initialization(&self, bean: Arc<dyn Any + Send + Sync>, bean_name: &str) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(bean))
    }
}
