//! ApplicationContext trait — Spring 风格的应用上下文。
use std::any::Any;
use std::sync::Arc;

/// 应用上下文 trait。
pub trait ApplicationContext: Send + Sync + 'static {
    fn get_bean(&self, name: &str) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;
    fn get_display_name(&self) -> &str;
    fn get_startup_date(&self) -> u64;
    fn is_active(&self) -> bool;
}
