//! DestructionAwareBeanPostProcessor — 销毁感知后处理器。
use crate::bean_post_processor::BeanPostProcessor;

/// 销毁感知后处理器 trait。
pub trait DestructionAwareBeanPostProcessor: BeanPostProcessor {
    fn requires_destruction(&self, bean: &dyn std::any::Any) -> bool;
    fn post_process_before_destruction(&self, bean: &dyn std::any::Any, bean_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}
