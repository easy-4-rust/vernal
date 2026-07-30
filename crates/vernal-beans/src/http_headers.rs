//! HttpHeaders — HTTP 头。
use std::collections::HashMap;

/// HTTP 头。
#[derive(Clone, Debug, Default)]
pub struct HttpHeaders {
    headers: HashMap<String, Vec<String>>,
}
impl HttpHeaders {
    pub fn new() -> Self { Self::default() }
    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.headers.entry(name.into()).or_default().push(value.into());
    }
    pub fn get(&self, name: &str) -> Option<&Vec<String>> { self.headers.get(name) }
    pub fn get_first(&self, name: &str) -> Option<&str> { self.headers.get(name)?.first().map(|s| s.as_str()) }
    pub fn contains(&self, name: &str) -> bool { self.headers.contains_key(name) }
    pub fn remove(&mut self, name: &str) -> Option<Vec<String>> { self.headers.remove(name) }
}
