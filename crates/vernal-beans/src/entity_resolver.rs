//! EntityResolver — XML 实体解析器 trait。
use std::fmt;

/// 解析后的实体。
#[derive(Clone, Debug)]
pub struct ResolvedEntity {
    pub entity_id: String,
    pub content: Vec<u8>,
}

/// 实体解析器 trait。
pub trait EntityResolver: fmt::Debug + Send + Sync {
    fn resolve_entity(&self, public_id: &str, system_id: &str) -> Option<ResolvedEntity>;
}
