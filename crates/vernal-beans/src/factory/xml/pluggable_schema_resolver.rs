//! PluggableSchemaResolver — 可插拔 Schema 解析器。
use crate::factory::xml::entity_resolver::{EntityResolver, ResolvedEntity};

/// 可插拔 Schema 解析器。
#[derive(Clone, Debug, Default)]
pub struct PluggableSchemaResolver {
    /// pub。
    pub schema_mappings: std::collections::HashMap<String, String>,
}
impl PluggableSchemaResolver {
    /// 创建一个新的实例。
    pub fn new() -> Self { Self::default() }
    /// 转换为text。
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
    fn resolve_entity(
        &self,
        _public_id: Option<&str>,
        system_id: &str,
    ) -> Result<ResolvedEntity, Box<dyn std::error::Error + Send + Sync>> {
        self.schema_mappings.get(system_id)
            .map(|path| ResolvedEntity {
                public_id: None,
                system_id: system_id.to_string(),
                content: path.as_bytes().to_vec(),
            })
            .ok_or_else(|| format!("No schema mapping found for: {}", system_id).into())
    }
}
