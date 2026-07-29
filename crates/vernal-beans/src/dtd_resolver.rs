//! DtdResolver — DTD 实体解析器。
use crate::entity_resolver::{EntityResolver, ResolvedEntity};

/// DTD 实体解析器。
#[derive(Clone, Debug)]
pub struct DtdResolver;
impl DtdResolver {
    pub fn new() -> Self { Self }
}
impl EntityResolver for DtdResolver {
    fn resolve_entity(&self, _public_id: &str, _system_id: &str) -> Option<ResolvedEntity> {
        None
    }
}
