//! QualifierAnnotationAutowireCandidateResolver — Spring 风格的限定符解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.QualifierAnnotationAutowireCandidateResolver`。
//!
//! 根据 `@Qualifier` 注解判断候选 Bean。
//!
//! 在 Spring 中，当容器中有多个同类型 Bean 时，`@Qualifier` 注解
//! 用于精确指定要注入的 Bean。此类负责匹配 `@Qualifier` 值与 Bean 的限定符。

use std::any::TypeId;
use std::sync::Mutex;
use std::collections::HashMap;

use crate::Qualifier;

/// Spring 风格的 `@Qualifier` 注解解析器。
///
/// 对应 Spring 的 `QualifierAnnotationAutowireCandidateResolver`。
///
/// 判断某个 Bean 是否匹配 `@Qualifier` 注解，实现精确的自动装配匹配。
pub struct QualifierAnnotationAutowireCandidateResolver {
    /// 限定符缓存：类型 -> 限定符列表
    qualifier_cache: Mutex<HashMap<TypeId, Vec<String>>>,
    /// 默认值
    default_qualifier: Mutex<Option<Qualifier>>,
}

impl QualifierAnnotationAutowireCandidateResolver {
    /// 创建新的限定符解析器。
    pub fn new() -> Self {
        Self {
            qualifier_cache: Mutex::new(HashMap::new()),
            default_qualifier: Mutex::new(None),
        }
    }

    /// 为指定类型注册一个限定符。
    pub fn register_qualifier(&self, type_id: TypeId, qualifier: String) {
        self.qualifier_cache.lock().unwrap()
            .entry(type_id)
            .or_default()
            .push(qualifier);
    }

    /// 获取指定类型的所有限定符。
    pub fn get_qualifiers(&self, type_id: TypeId) -> Vec<String> {
        self.qualifier_cache.lock().unwrap()
            .get(&type_id)
            .cloned()
            .unwrap_or_default()
    }

    /// 指定类型的限定符数量。
    pub fn qualifier_count(&self, type_id: TypeId) -> usize {
        self.qualifier_cache.lock().unwrap()
            .get(&type_id)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// 设置默认限定符（当 Bean 没有显式限定符时使用）。
    pub fn set_default_qualifier(&self, qualifier: Qualifier) {
        *self.default_qualifier.lock().unwrap() = Some(qualifier);
    }

    /// 获取默认限定符。
    pub fn default_qualifier(&self) -> Option<Qualifier> {
        self.default_qualifier.lock().unwrap().clone()
    }

    /// 判断指定类型是否包含指定限定符。
    pub fn has_qualifier(&self, type_id: TypeId, qualifier: &str) -> bool {
        self.qualifier_cache.lock().unwrap()
            .get(&type_id)
            .map(|v| v.iter().any(|q| q == qualifier))
            .unwrap_or(false)
    }

    /// 判断指定类型是否为自动装配候选（至少有一个限定符）。
    pub fn is_autowire_candidate(&self, type_id: TypeId) -> bool {
        self.qualifier_cache.lock().unwrap()
            .get(&type_id)
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }
}

impl Default for QualifierAnnotationAutowireCandidateResolver {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_check_qualifier() {
        let resolver = QualifierAnnotationAutowireCandidateResolver::new();
        let tid = TypeId::of::<String>();

        resolver.register_qualifier(tid, "primary".to_string());
        resolver.register_qualifier(tid, "myQualifier".to_string());

        assert!(resolver.has_qualifier(tid, "primary"));
        assert!(resolver.has_qualifier(tid, "myQualifier"));
        assert!(!resolver.has_qualifier(tid, "missing"));
        assert_eq!(resolver.qualifier_count(tid), 2);
    }

    #[test]
    fn is_autowire_candidate_with_and_without_qualifiers() {
        let resolver = QualifierAnnotationAutowireCandidateResolver::new();
        let tid_a = TypeId::of::<String>();
        let tid_b = TypeId::of::<i32>();

        resolver.register_qualifier(tid_a, "q".to_string());

        assert!(resolver.is_autowire_candidate(tid_a));
        assert!(!resolver.is_autowire_candidate(tid_b));
    }

    #[test]
    fn default_qualifier() {
        let resolver = QualifierAnnotationAutowireCandidateResolver::new();
        assert!(resolver.default_qualifier().is_none());
        resolver.set_default_qualifier(Qualifier::new("defaultQ".to_string()).unwrap());
        assert!(resolver.default_qualifier().is_some());
    }

    #[test]
    fn get_qualifiers_returns_all() {
        let resolver = QualifierAnnotationAutowireCandidateResolver::new();
        let tid = TypeId::of::<u64>();
        resolver.register_qualifier(tid, "a".to_string());
        resolver.register_qualifier(tid, "b".to_string());
        let mut qs = resolver.get_qualifiers(tid);
        qs.sort();
        assert_eq!(qs, vec!["a", "b"]);
    }
}
