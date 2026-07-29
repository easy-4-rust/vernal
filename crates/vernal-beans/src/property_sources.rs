//! PropertySources — Spring 风格的属性源集合。
use crate::property_source::PropertySource;

/// 属性源集合。
#[derive(Clone, Debug, Default)]
pub struct PropertySources { pub sources: Vec<PropertySource> }
impl PropertySources {
    pub fn new() -> Self { Self::default() }
    pub fn add(&mut self, source: PropertySource) { self.sources.push(source); }
    pub fn get(&self, name: &str) -> Option<&PropertySource> { self.sources.iter().find(|s| s.name() == name) }
    pub fn get_mut(&mut self, name: &str) -> Option<&mut PropertySource> { self.sources.iter_mut().find(|s| s.name() == name) }
    pub fn contains(&self, name: &str) -> bool { self.sources.iter().any(|s| s.name() == name) }
    pub fn names(&self) -> Vec<&str> { self.sources.iter().map(|s| s.name()).collect() }
    pub fn len(&self) -> usize { self.sources.len() }
    pub fn is_empty(&self) -> bool { self.sources.is_empty() }
    pub fn clear(&mut self) { self.sources.clear(); }
}
