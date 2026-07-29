//! PluggableSchemaResolver — 可插拔 Schema 解析器。
use crate::entity_resolver::{EntityResolver, ResolvedEntity};

/// 可插拔 Schema 解析器。
#[derive(Clone, Debug, Default)]
pub struct PluggableSchemaResolver {
    pub schema_mappings: std::collections::HashMap<String, String>,
}
impl PluggableSchemaResolver {
    pub fn new() -> Self { Self::default() }
    pub fn from_text(mappings_text: &str) -> Self {
        let mut schema_mappings = std::collections::HashMap::new();
        for line in mappings_text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            if let Some((k, v)) = line.split_once('=') {
                schema_mappings.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
        Self { schema_mappings }
    }
}
impl EntityResolver for PluggableSchemaResolver {
    fn resolve_entity(&self, _public_id: &str, system_id: &str) -> Option<ResolvedEntity> {
        self.schema_mappings.get(system_id).map(|path| ResolvedEntity {
            entity_id: system_id.to_string(),
            content: path.as_bytes().to_vec(),
        })
    }
}
