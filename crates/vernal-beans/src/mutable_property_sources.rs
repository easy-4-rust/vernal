//! MutablePropertySources — 可变属性源集合。
use crate::property_source::PropertySource;
use crate::property_sources::PropertySources;

/// 可变属性源集合。
#[derive(Clone, Debug, Default)]
pub struct MutablePropertySources { inner: PropertySources }
impl MutablePropertySources {
    pub fn new() -> Self { Self::default() }
    pub fn add_first(&mut self, source: PropertySource) { self.inner.sources.insert(0, source); }
    pub fn add_last(&mut self, source: PropertySource) { self.inner.add(source); }
    pub fn add_before(&mut self, before: &str, source: PropertySource) {
        if let Some(idx) = self.inner.sources.iter().position(|s| s.name() == before) {
            self.inner.sources.insert(idx, source);
        } else {
            self.inner.add(source);
        }
    }
    pub fn add_after(&mut self, after: &str, source: PropertySource) {
        if let Some(idx) = self.inner.sources.iter().position(|s| s.name() == after) {
            self.inner.sources.insert(idx + 1, source);
        } else {
            self.inner.add(source);
        }
    }
    pub fn remove(&mut self, name: &str) -> Option<PropertySource> {
        self.inner.sources.iter().position(|s| s.name() == name)
            .map(|i| self.inner.sources.remove(i))
    }
    pub fn replace(&mut self, name: &str, source: PropertySource) -> Option<PropertySource> {
        self.inner.sources.iter().position(|s| s.name() == name)
            .map(|i| std::mem::replace(&mut self.inner.sources[i], source))
    }
    pub fn as_inner(&self) -> &PropertySources { &self.inner }
}
