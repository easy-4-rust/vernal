//! ManagedProperties — Spring 风格的托管属性。
#[derive(Clone, Debug, Default)]
pub struct ManagedProperties { items: std::collections::HashMap<String, String> }
impl ManagedProperties {
    pub fn new() -> Self { Self::default() }
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) { self.items.insert(key.into(), value.into()); }
    pub fn get(&self, key: &str) -> Option<&str> { self.items.get(key).map(|s| s.as_str()) }
    pub fn contains_key(&self, key: &str) -> bool { self.items.contains_key(key) }
    pub fn len(&self) -> usize { self.items.len() }
    pub fn is_empty(&self) -> bool { self.items.is_empty() }
}
