//! DisposableBeanAdapter — 可销毁 Bean 适配器。
use std::any::Any;
use std::sync::Arc;

/// 可销毁 Bean 适配器。
#[derive(Clone, Debug)]
pub struct DisposableBeanAdapter {
    pub bean_name: String,
}
impl DisposableBeanAdapter {
    pub fn new(bean_name: impl Into<String>) -> Self {
        Self { bean_name: bean_name.into() }
    }
    pub fn bean_name(&self) -> &str { &self.bean_name }
    pub fn destroy(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}
