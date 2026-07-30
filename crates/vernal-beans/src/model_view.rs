//! ModelView — 模型视图。
use std::collections::HashMap;

/// 模型视图。
#[derive(Debug, Default)]
pub struct ModelView {
    pub view_name: String,
    pub model: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}
impl ModelView {
    pub fn new(view_name: impl Into<String>) -> Self { Self { view_name: view_name.into(), model: HashMap::new() } }
    pub fn view_name(&self) -> &str { &self.view_name }
    pub fn add_object(&mut self, key: impl Into<String>, value: Box<dyn std::any::Any + Send + Sync>) {
        self.model.insert(key.into(), value);
    }
    pub fn get_object(&self, key: &str) -> Option<&(dyn std::any::Any + Send + Sync)> {
        self.model.get(key).map(|v| v.as_ref())
    }
    pub fn is_empty(&self) -> bool { self.model.is_empty() }
    pub fn size(&self) -> usize { self.model.len() }
}
