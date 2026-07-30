//! DtdResolverImpl — DTD 实体解析器实现。
use crate::entity_resolver::{EntityResolver, ResolvedEntity};

/// DTD 实体解析器实现。
#[derive(Clone, Debug, Default)]
pub struct DtdResolverImpl;
impl DtdResolverImpl {
    pub fn new() -> Self { Self }
}
impl EntityResolver for DtdResolverImpl {
    fn resolve_entity(&self, _public_id: &str, _system_id: &str) -> Option<ResolvedEntity> {
        None
    }
}
