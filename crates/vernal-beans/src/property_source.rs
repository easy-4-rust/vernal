//! PropertySource — Spring 风格的属性源。
#[derive(Clone, Debug)]
pub struct PropertySource {
    pub name: String,
    pub source: std::collections::HashMap<String, String>,
}
impl PropertySource {
    pub fn new(name: impl Into<String>) -> Self { Self { name: name.into(), source: std::collections::HashMap::new() } }
    pub fn name(&self) -> &str { &self.name }
    pub fn get_property(&self, key: &str) -> Option<&str> { self.source.get(key).map(|s| s.as_str()) }
    pub fn contains_property(&self, key: &str) -> bool { self.source.contains_key(key) }
    pub fn property_names(&self) -> Vec<&str> { self.source.keys().map(|s| s.as_str()).collect() }
    pub fn len(&self) -> usize { self.source.len() }
    pub fn is_empty(&self) -> bool { self.source.is_empty() }
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) { self.source.insert(key.into(), value.into()); }
    pub fn remove(&mut self, key: &str) -> Option<String> { self.source.remove(key) }
}
