//! AbstractBeanFactory — 抽象 Bean 工厂。
use std::any::Any;
use std::sync::Arc;

/// 抽象 Bean 工厂。
#[derive(Clone, Debug, Default)]
pub struct AbstractBeanFactory;
impl AbstractBeanFactory {
    pub fn new() -> Self { Self }
    pub fn get_bean_by_type(&self, _type_id: std::any::TypeId) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Err("Not implemented".into())
    }
}
