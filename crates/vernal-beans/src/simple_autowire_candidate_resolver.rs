//! SimpleAutowireCandidateResolver — 简单自动装配候选解析器。
use std::any::TypeId;

/// 简单自动装配候选解析器。
#[derive(Clone, Debug, Default)]
pub struct SimpleAutowireCandidateResolver;
impl SimpleAutowireCandidateResolver {
    pub fn new() -> Self { Self }
    pub fn is_autowire_candidate(&self, _type_id: TypeId) -> bool { true }
}
