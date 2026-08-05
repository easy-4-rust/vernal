//! CachedIntrospectionResults — 对应 Spring `org.springframework.beans.CachedIntrospectionResults`。
//!
//! 缓存的内省结果。

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::RwLock;

/// 缓存的内省结果。
///
/// 对应 Java 类：`org.springframework.beans.CachedIntrospectionResults`。
///
/// 缓存 Bean 的内省结果，避免重复反射。
#[derive(Debug)]
pub struct CachedIntrospectionResults {
    /// 类型到属性描述符的映射。
    class_cache: RwLock<HashMap<TypeId, HashMap<String, PropertyDescriptorEntry>>>,
}

/// 属性描述符条目。
#[derive(Debug, Clone)]
pub struct PropertyDescriptorEntry {
    /// 属性名称。
    name: String,
    /// 属性类型。
    property_type: TypeId,
    /// 是否可读。
    readable: bool,
    /// 是否可写。
    writable: bool,
}

impl PropertyDescriptorEntry {
    /// 创建一个新的 PropertyDescriptorEntry。
    pub fn new(
        name: impl Into<String>,
        property_type: TypeId,
        readable: bool,
        writable: bool,
    ) -> Self {
        Self {
            name: name.into(),
            property_type,
            readable,
            writable,
        }
    }

    /// 获取属性名称。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取属性类型。
    pub fn property_type(&self) -> TypeId {
        self.property_type
    }

    /// 是否可读。
    pub fn is_readable(&self) -> bool {
        self.readable
    }

    /// 是否可写。
    pub fn is_writable(&self) -> bool {
        self.writable
    }
}

impl CachedIntrospectionResults {
    /// 创建一个新的 CachedIntrospectionResults。
    pub fn new() -> Self {
        Self {
            class_cache: RwLock::new(HashMap::new()),
        }
    }

    /// 注册属性描述符。
    pub fn register_property(&self, type_id: TypeId, descriptor: PropertyDescriptorEntry) {
        if let Ok(mut cache) = self.class_cache.write() {
            let type_cache = cache.entry(type_id).or_insert_with(HashMap::new);
            type_cache.insert(descriptor.name.clone(), descriptor);
        }
    }

    /// 获取属性描述符。
    pub fn get_property_descriptor(
        &self,
        type_id: TypeId,
        name: &str,
    ) -> Option<PropertyDescriptorEntry> {
        self.class_cache.read().ok().and_then(|cache| {
            cache
                .get(&type_id)
                .and_then(|type_cache| type_cache.get(name).cloned())
        })
    }

    /// 获取类型的所有属性描述符。
    pub fn get_property_descriptors(&self, type_id: TypeId) -> Vec<PropertyDescriptorEntry> {
        self.class_cache
            .read()
            .ok()
            .and_then(|cache| {
                cache
                    .get(&type_id)
                    .map(|type_cache| type_cache.values().cloned().collect())
            })
            .unwrap_or_default()
    }

    /// 检查是否有指定类型的缓存。
    pub fn has_cached_class(&self, type_id: TypeId) -> bool {
        self.class_cache
            .read()
            .ok()
            .map(|cache| cache.contains_key(&type_id))
            .unwrap_or(false)
    }

    /// 清除所有缓存。
    pub fn clear(&self) {
        if let Ok(mut cache) = self.class_cache.write() {
            cache.clear();
        }
    }
}

impl Default for CachedIntrospectionResults {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let results = CachedIntrospectionResults::new();
        assert!(!results.has_cached_class(TypeId::of::<String>()));
    }

    #[test]
    fn test_register_and_get() {
        let results = CachedIntrospectionResults::new();
        let desc = PropertyDescriptorEntry::new("name", TypeId::of::<String>(), true, true);
        results.register_property(TypeId::of::<String>(), desc);

        let retrieved = results
            .get_property_descriptor(TypeId::of::<String>(), "name")
            .unwrap();
        assert_eq!(retrieved.name(), "name");
        assert!(retrieved.is_readable());
        assert!(retrieved.is_writable());
    }

    #[test]
    fn test_get_property_descriptors() {
        let results = CachedIntrospectionResults::new();
        results.register_property(
            TypeId::of::<String>(),
            PropertyDescriptorEntry::new("a", TypeId::of::<i32>(), true, true),
        );
        results.register_property(
            TypeId::of::<String>(),
            PropertyDescriptorEntry::new("b", TypeId::of::<bool>(), true, false),
        );

        let descs = results.get_property_descriptors(TypeId::of::<String>());
        assert_eq!(descs.len(), 2);
    }

    #[test]
    fn test_clear() {
        let results = CachedIntrospectionResults::new();
        results.register_property(
            TypeId::of::<String>(),
            PropertyDescriptorEntry::new("name", TypeId::of::<String>(), true, true),
        );
        assert!(results.has_cached_class(TypeId::of::<String>()));

        results.clear();
        assert!(!results.has_cached_class(TypeId::of::<String>()));
    }
}
