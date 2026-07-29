//! PropertyResolver trait — Spring 风格的属性解析器。
use std::fmt;

/// 属性解析器 trait。
pub trait PropertyResolver: Send + Sync + fmt::Debug {
    fn get_property(&self, key: &str) -> Option<String>;
    fn get_required_property(&self, key: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    fn contains_property(&self, key: &str) -> bool;
    fn resolve_placeholders(&self, text: &str) -> String;
    fn resolve_required_placeholders(&self, text: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}
