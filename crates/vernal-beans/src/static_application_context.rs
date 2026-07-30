//! StaticApplicationContext — 静态应用上下文。
use crate::application_context::ApplicationContext;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 静态应用上下文。
#[derive(Clone, Debug, Default)]
pub struct StaticApplicationContext {
    pub display_name: String,
    beans: Arc<Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>>,
}
impl StaticApplicationContext {
    pub fn new(display_name: impl Into<String>) -> Self { Self { display_name: display_name.into(), beans: Arc::new(Mutex::new(HashMap::new())) } }
    pub fn register_bean(&self, name: impl Into<String>, bean: Arc<dyn Any + Send + Sync>) { self.beans.lock().unwrap().insert(name.into(), bean); }
}
impl ApplicationContext for StaticApplicationContext {
    fn get_bean(&self, name: &str) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        self.beans.lock().unwrap().get(name).cloned().ok_or_else(|| format!("Bean '{}' not found", name).into())
    }
    fn get_display_name(&self) -> &str { &self.display_name }
    fn get_startup_date(&self) -> u64 { 0 }
    fn is_active(&self) -> bool { true }
}
