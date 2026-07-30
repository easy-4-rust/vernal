//! AbstractAutowireCapableBeanFactory — 抽象自动装配 Bean 工厂。
use std::any::Any;
use std::sync::Arc;

/// 抽象自动装配 Bean 工厂。
#[derive(Clone, Debug, Default)]
pub struct AbstractAutowireCapableBeanFactory;
impl AbstractAutowireCapableBeanFactory {
    pub fn new() -> Self { Self }
    pub fn create_bean(&self, _bean_name: &str, _bean_class_name: &str) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Err("Not implemented".into())
    }
}
