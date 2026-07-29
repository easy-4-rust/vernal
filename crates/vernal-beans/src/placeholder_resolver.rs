//! PlaceholderResolver — 占位符解析器。
use crate::property_source::PropertySource;

/// 占位符解析器。
#[derive(Clone, Debug, Default)]
pub struct PlaceholderResolver {
    pub sources: Vec<PropertySource>,
}
impl PlaceholderResolver {
    pub fn new() -> Self { Self::default() }
    pub fn add_source(&mut self, source: PropertySource) { self.sources.push(source); }
    pub fn resolve(&self, text: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut result = text.to_string();
        for source in &self.sources {
            for name in source.property_names() {
                if let Some(val) = source.get_property(name) {
                    result = result.replace(&format!("${{{}}}", name), val);
                }
            }
        }
        Ok(result)
    }
}
