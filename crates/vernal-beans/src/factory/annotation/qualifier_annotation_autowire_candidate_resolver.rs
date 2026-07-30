//! QualifierAnnotationAutowireCandidateResolver — Spring 风格的限定符解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.QualifierAnnotationAutowireCandidateResolver`。
//!
//! 根据 `@Qualifier` 注解判断候选 Bean。

use std::any::TypeId;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::Qualifier;

/// Spring 风格的 `@Qualifier` 注解解析器。
///
/// 对应 Spring 的 `QualifierAnnotationAutowireCandidateResolver`。
///
/// 判断某个 Bean 是否匹配 `@Qualifier` 注解。
pub struct QualifierAnnotationAutowireCandidateResolver {
    /// 限定符缓存：类型 -> 限定符列表
    qualifier_cache: Mutex<HashMap<TypeId, Vec<String>>>,
    /// 默认值
    default_qualifier: Mutex<Option<Qualifier>>,
}

impl QualifierAnnotationAutowireCandidateResolver {
    pub fn new() -> Self {
        Self {
            qualifier_cache: Mutex::new(HashMap::new()),
            default_qualifier: Mutex::new(None),
        }
    }

    pub fn register_qualifier(&self, type_id: TypeId, qualifier: String) {
        self.qualifier_cache.lock().unwrap()
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(qualifier);
    }

    pub fn get_qualifiers(&self, type_id: TypeId) -> Vec<String> {
        self.qualifier_cache.lock().unwrap()
            .get(&type_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn qualifier_count(&self, type_id: TypeId) -> usize {
        self.qualifier_cache.lock().unwrap()
            .get(&type_id)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    pub fn set_default_qualifier(&self, qualifier: Qualifier) {
        *self.default_qualifier.lock().unwrap() = Some(qualifier);
    }

    pub fn has_qualifier(&self, type_id: TypeId, qualifier: &str) -> bool {
        self.qualifier_cache.lock().unwrap()
            .get(&type_id)
            .map(|v| v.iter().any(|q| q == qualifier))
            .unwrap_or(false)
    }

    pub fn is_autowire_candidate(&self, type_id: TypeId) -> bool {
        !self.qualifier_cache.lock().unwrap().get(&type_id).map(|v| v.is_empty()).unwrap_or(true)
    }
}

impl Default for QualifierAnnotationAutowireCandidateResolver {
    fn default() -> Self { Self::new() }
}
