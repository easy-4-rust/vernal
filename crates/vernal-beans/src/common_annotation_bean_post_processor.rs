//! CommonAnnotationBeanPostProcessor — 通用注解后处理器。
use crate::factory::config::bean_post_processor::BeanPostProcessor;
use std::any::Any;
use std::sync::Arc;

/// 通用注解后处理器。
#[derive(Clone, Debug, Default)]
pub struct CommonAnnotationBeanPostProcessor;
impl CommonAnnotationBeanPostProcessor {
    pub fn new() -> Self { Self }
}
impl BeanPostProcessor for CommonAnnotationBeanPostProcessor {
    fn post_process_before_initialization(&self, bean: Arc<dyn Any + Send + Sync>, _name: &str) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }
    fn post_process_after_initialization(&self, bean: Arc<dyn Any + Send + Sync>, _name: &str) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }
}
