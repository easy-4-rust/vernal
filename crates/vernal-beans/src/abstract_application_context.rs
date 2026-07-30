//! AbstractApplicationContext — 抽象应用上下文。
use crate::application_context::ApplicationContext;
use std::any::Any;
use std::sync::Arc;

/// 抽象应用上下文。
#[derive(Clone, Debug, Default)]
pub struct AbstractApplicationContext {
    pub display_name: String,
    pub startup_date: u64,
    pub active: bool,
}
impl AbstractApplicationContext {
    pub fn new(display_name: impl Into<String>) -> Self {
        Self { display_name: display_name.into(), startup_date: 0, active: false }
    }
    pub fn refresh(&mut self) { self.active = true; self.startup_date = now_ms(); }
    pub fn close(&mut self) { self.active = false; }
}
fn now_ms() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64 }
impl ApplicationContext for AbstractApplicationContext {
    fn get_bean(&self, _name: &str) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> { Err("Not implemented".into()) }
    fn get_display_name(&self) -> &str { &self.display_name }
    fn get_startup_date(&self) -> u64 { self.startup_date }
    fn is_active(&self) -> bool { self.active }
}
