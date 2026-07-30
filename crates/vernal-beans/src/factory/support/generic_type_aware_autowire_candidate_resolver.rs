//! GenericTypeAwareAutowireCandidateResolver — Spring 风格泛型感知自动装配解析器。

use std::any::TypeId;
use std::collections::HashSet;
use std::sync::Mutex;

use crate::factory::support::autowire_candidate_resolver::AutowireCandidateResolver;

pub struct GenericTypeAwareAutowireCandidateResolver {
    excluded_types: Mutex<HashSet<TypeId>>,
}

impl GenericTypeAwareAutowireCandidateResolver {
    pub fn new() -> Self { Self { excluded_types: Mutex::new(HashSet::new()) } }
    pub fn exclude_type(&self, type_id: TypeId) {
        self.excluded_types.lock().unwrap().insert(type_id);
    }
    pub fn include_type(&self, type_id: TypeId) {
        self.excluded_types.lock().unwrap().remove(&type_id);
    }
    pub fn excluded_count(&self) -> usize {
        self.excluded_types.lock().unwrap().len()
    }
}

impl AutowireCandidateResolver for GenericTypeAwareAutowireCandidateResolver {
    fn is_autowire_candidate(&self, type_id: TypeId, _name: &str) -> bool {
        !self.excluded_types.lock().unwrap().contains(&type_id)
    }
}

impl Default for GenericTypeAwareAutowireCandidateResolver { fn default() -> Self { Self::new() } }
