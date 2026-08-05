//! GenericTypeAwareAutowireCandidateResolver — Spring 风格泛型感知自动装配解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.GenericTypeAwareAutowireCandidateResolver`。
//!
//! 通过维护一个排除类型列表来判断候选 Bean 是否可自动装配。
//! 被排除的类型不会被视为自动装配候选，其余类型均可。

use std::any::TypeId;
use std::collections::HashSet;
use std::sync::Mutex;

use crate::factory::support::autowire_candidate_resolver::AutowireCandidateResolver;

/// Spring 风格泛型感知自动装配候选解析器。
///
/// 对应 Spring 的 `GenericTypeAwareAutowireCandidateResolver`。
///
/// 通过维护排除类型集来控制哪些类型不可自动装配。
pub struct GenericTypeAwareAutowireCandidateResolver {
    excluded_types: Mutex<HashSet<TypeId>>,
}

impl GenericTypeAwareAutowireCandidateResolver {
    /// 创建新的解析器（默认不排除任何类型）。
    pub fn new() -> Self {
        Self {
            excluded_types: Mutex::new(HashSet::new()),
        }
    }

    /// 将指定类型排除出自动装配候选。
    pub fn exclude_type(&self, type_id: TypeId) {
        self.excluded_types.lock().unwrap().insert(type_id);
    }

    /// 将指定类型重新纳入自动装配候选。
    pub fn include_type(&self, type_id: TypeId) {
        self.excluded_types.lock().unwrap().remove(&type_id);
    }

    /// 当前被排除的类型数量。
    pub fn excluded_count(&self) -> usize {
        self.excluded_types.lock().unwrap().len()
    }

    /// 判断指定类型是否被排除。
    pub fn is_excluded(&self, type_id: TypeId) -> bool {
        self.excluded_types.lock().unwrap().contains(&type_id)
    }
}

impl AutowireCandidateResolver for GenericTypeAwareAutowireCandidateResolver {
    fn is_autowire_candidate(&self, type_id: TypeId, _name: &str) -> bool {
        !self.excluded_types.lock().unwrap().contains(&type_id)
    }
}

impl Default for GenericTypeAwareAutowireCandidateResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_allows_all_types() {
        let resolver = GenericTypeAwareAutowireCandidateResolver::new();
        assert!(resolver.is_autowire_candidate(TypeId::of::<String>(), "strBean"));
        assert!(resolver.is_autowire_candidate(TypeId::of::<i32>(), "intBean"));
        assert_eq!(resolver.excluded_count(), 0);
    }

    #[test]
    fn exclude_and_include_type() {
        let resolver = GenericTypeAwareAutowireCandidateResolver::new();
        let tid = TypeId::of::<String>();

        resolver.exclude_type(tid);
        assert!(!resolver.is_autowire_candidate(tid, "strBean"));
        assert!(resolver.is_excluded(tid));
        assert_eq!(resolver.excluded_count(), 1);

        resolver.include_type(tid);
        assert!(resolver.is_autowire_candidate(tid, "strBean"));
        assert!(!resolver.is_excluded(tid));
        assert_eq!(resolver.excluded_count(), 0);
    }

    #[test]
    fn only_excluded_type_is_rejected() {
        let resolver = GenericTypeAwareAutowireCandidateResolver::new();
        resolver.exclude_type(TypeId::of::<i32>());

        assert!(resolver.is_autowire_candidate(TypeId::of::<String>(), "s"));
        assert!(!resolver.is_autowire_candidate(TypeId::of::<i32>(), "i"));
    }
}
