//! AutowireUtils — 自动装配工具。
use std::any::TypeId;

/// 自动装配工具。
#[derive(Clone, Debug)]
pub struct AutowireUtils;
impl AutowireUtils {
    pub fn is_autowire_candidate(type_id: TypeId, candidate_type_id: TypeId) -> bool {
        type_id == candidate_type_id
    }
    pub fn determine_autowire_candidates(type_id: TypeId, candidates: &[TypeId]) -> Vec<TypeId> {
        candidates.iter().filter(|&&c| c == type_id).cloned().collect()
    }
}
