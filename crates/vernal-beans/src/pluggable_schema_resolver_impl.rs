//! PluggableSchemaResolverImpl — 可插拔 Schema 解析器实现。
use crate::factory::xml::entity_resolver::{EntityResolver, ResolvedEntity};

/// 可插拔 Schema 解析器实现。
#[derive(Clone, Debug, Default)]
pub struct PluggableSchemaResolverImpl {
    pub schema_mappings: std::collections::HashMap<String, String>,
}
impl PluggableSchemaResolverImpl {
    pub fn new() -> Self { Self::default() }
    pub fn from_text(mappings: &str) -> Self {
        let mut m = std::collections::HashMap::new();
        for line in mappings.lines() {
            if let Some((k, v)) = line.split_once('=') {
                m.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
        Self { schema_mappings: m }
    }
}
impl EntityResolver for PluggableSchemaResolverImpl {
    fn resolve_entity(&self, _public_id: &str, system_id: &str) -> Option<ResolvedEntity> {
        self.schema_mappings.get(system_id).map(|path| ResolvedEntity {
            entity_id: system_id.to_string(),
            content: path.as_bytes().to_vec(),
        })
    }
}
