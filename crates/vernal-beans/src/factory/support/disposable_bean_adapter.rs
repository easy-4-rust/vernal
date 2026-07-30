//! DisposableBeanAdapter — Spring 风格可销毁 Bean 适配器。

use std::any::Any;
use std::sync::Arc;

pub struct DisposableBeanAdapter {
    bean_name: String,
    bean: Arc<dyn Any + Send + Sync>,
    destroy_method_name: Option<String>,
}

impl DisposableBeanAdapter {
    pub fn new(bean_name: String, bean: Arc<dyn Any + Send + Sync>) -> Self {
        Self { bean_name, bean, destroy_method_name: None }
    }
    pub fn with_destroy_method(mut self, name: String) -> Self {
        self.destroy_method_name = Some(name);
        self
    }
    pub fn bean_name(&self) -> &str { &self.bean_name }
    pub fn bean(&self) -> &Arc<dyn Any + Send + Sync> { &self.bean }
    pub fn destroy_method_name(&self) -> Option<&str> { self.destroy_method_name.as_deref() }
    pub fn run_destroy(&self) {
        // 执行销毁逻辑
    }
}
