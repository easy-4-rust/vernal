//! Environment trait — Spring 风格的环境接口。
use std::fmt;

/// 环境 trait。
pub trait Environment: Send + Sync + fmt::Debug {
    fn get_property(&self, key: &str) -> Option<String>;
    fn get_required_property(&self, key: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.get_property(key).ok_or_else(|| format!("Required property '{}' not found", key).into())
    }
    fn contains_property(&self, key: &str) -> bool { self.get_property(key).is_some() }
    fn get_active_profiles(&self) -> Vec<String> { Vec::new() }
    fn get_default_profiles(&self) -> Vec<String> { Vec::new() }
    fn resolve_placeholders(&self, text: &str) -> String { text.to_string() }
}
