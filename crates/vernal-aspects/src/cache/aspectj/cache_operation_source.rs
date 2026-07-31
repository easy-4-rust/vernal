//! 对标 `org.springframework.cache.interceptor.CacheOperationSource` 接口。
//!
//! 缓存操作源：从方法元数据中获取缓存操作。

use super::cache_operation::CacheOperationMetadata;

/// 方法元数据（缓存模块用）。
///
/// 对标 Spring 的 `java.lang.reflect.Method` + `Class<?>` 元组。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MethodMetadata {
    /// 方法所属类型的完全限定名。
    pub type_name: &'static str,
    /// 方法名。
    pub method_name: &'static str,
    /// 方法参数类型名列表。
    pub parameter_types: Vec<&'static str>,
}

impl MethodMetadata {
    /// 创建新的方法元数据。
    pub fn new(type_name: &'static str, method_name: &'static str) -> Self {
        Self {
            type_name,
            method_name,
            parameter_types: Vec::new(),
        }
    }

    /// 返回方法的完全限定签名。
    pub fn qualified_name(&self) -> String {
        let params = self.parameter_types.join(", ");
        format!("{}#{}({})", self.type_name, self.method_name, params)
    }
}

/// 缓存操作源 trait。
///
/// 对标 Spring 的 `CacheOperationSource` 接口。
pub trait CacheOperationSource: Send + Sync + 'static {
    /// 根据方法元数据获取缓存操作。
    fn get_cache_operation(&self, method: &MethodMetadata) -> Option<CacheOperationMetadata>;

    /// 是否存在缓存操作源。
    fn is_candidate_class(&self, type_name: &str) -> bool;
}

/// 基于注解的缓存操作源。
///
/// 对标 Spring 的 `AnnotationCacheOperationSource`。
#[allow(dead_code)] // Java 镜像脚手架：当前阶段未在切面中实际构造，供后续集成使用
pub struct AnnotationCacheOperationSource {
    /// 方法名 → 缓存操作元数据的映射。
    operations: std::collections::HashMap<String, CacheOperationMetadata>,
    /// 类名 → 缓存操作元数据的映射（类级默认）。
    class_operations: std::collections::HashMap<String, CacheOperationMetadata>,
}

#[allow(dead_code)] // Java 镜像脚手架：注册操作目前仅在测试中直接调用
impl AnnotationCacheOperationSource {
    /// 创建新的注解缓存操作源。
    pub fn new() -> Self {
        Self {
            operations: std::collections::HashMap::new(),
            class_operations: std::collections::HashMap::new(),
        }
    }

    /// 注册方法级缓存操作。
    pub fn register_method(&mut self, method_key: String, operation: CacheOperationMetadata) {
        self.operations.insert(method_key, operation);
    }

    /// 注册类级缓存操作。
    pub fn register_class(&mut self, type_name: String, operation: CacheOperationMetadata) {
        self.class_operations.insert(type_name, operation);
    }
}

impl Default for AnnotationCacheOperationSource {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheOperationSource for AnnotationCacheOperationSource {
    fn get_cache_operation(&self, method: &MethodMetadata) -> Option<CacheOperationMetadata> {
        if let Some(op) = self.operations.get(&method.qualified_name()) {
            return Some(op.clone());
        }
        self.class_operations.get(method.type_name).cloned()
    }

    fn is_candidate_class(&self, type_name: &str) -> bool {
        self.class_operations.contains_key(type_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::aspectj::cache_operation::{CacheOperation, CacheOperationMetadata};

    #[test]
    fn test_method_metadata() {
        let meta = MethodMetadata::new("com.example.Foo", "bar");
        assert_eq!(meta.qualified_name(), "com.example.Foo#bar()");
    }

    #[test]
    fn test_annotation_cache_operation_source_empty() {
        let source = AnnotationCacheOperationSource::new();
        let meta = MethodMetadata::new("com.example.Foo", "bar");
        assert!(source.get_cache_operation(&meta).is_none());
    }

    #[test]
    fn test_annotation_cache_operation_source_with_method() {
        let mut source = AnnotationCacheOperationSource::new();
        let op = CacheOperationMetadata {
            operation: CacheOperation::Cacheable,
            cache_names: vec![std::borrow::Cow::Borrowed("users")],
            ..Default::default()
        };
        source.register_method("com.example.Foo#bar()".to_string(), op);

        let meta = MethodMetadata::new("com.example.Foo", "bar");
        let found = source.get_cache_operation(&meta);
        assert!(found.is_some());
        assert_eq!(found.unwrap().operation, CacheOperation::Cacheable);
    }

    #[test]
    fn test_annotation_cache_operation_source_with_class() {
        let mut source = AnnotationCacheOperationSource::new();
        let op = CacheOperationMetadata {
            operation: CacheOperation::CachePut,
            cache_names: vec![std::borrow::Cow::Borrowed("users")],
            ..Default::default()
        };
        source.register_class("com.example.Foo".to_string(), op);

        assert!(source.is_candidate_class("com.example.Foo"));

        let meta = MethodMetadata::new("com.example.Foo", "bar");
        let found = source.get_cache_operation(&meta);
        assert!(found.is_some());
        assert_eq!(found.unwrap().operation, CacheOperation::CachePut);
    }

    #[test]
    fn test_annotation_cache_operation_source_method_overrides_class() {
        let mut source = AnnotationCacheOperationSource::new();

        // 类级操作
        let class_op = CacheOperationMetadata {
            operation: CacheOperation::CachePut,
            cache_names: vec![std::borrow::Cow::Borrowed("users")],
            ..Default::default()
        };
        source.register_class("com.example.Foo".to_string(), class_op);

        // 方法级操作（覆盖类级）
        let method_op = CacheOperationMetadata {
            operation: CacheOperation::Cacheable,
            cache_names: vec![std::borrow::Cow::Borrowed("users")],
            ..Default::default()
        };
        source.register_method("com.example.Foo#bar()".to_string(), method_op);

        let meta = MethodMetadata::new("com.example.Foo", "bar");
        let found = source.get_cache_operation(&meta);
        assert!(found.is_some());
        // 方法级覆盖类级
        assert_eq!(found.unwrap().operation, CacheOperation::Cacheable);
    }

    #[test]
    fn test_annotation_cache_operation_source_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<AnnotationCacheOperationSource>();
        assert_sync::<AnnotationCacheOperationSource>();
    }

    #[test]
    fn test_cache_operation_source_trait_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<AnnotationCacheOperationSource>();
        assert_sync::<AnnotationCacheOperationSource>();
    }

    #[test]
    fn test_annotation_cache_operation_source_default() {
        let source = AnnotationCacheOperationSource::default();
        let meta = MethodMetadata::new("Foo", "bar");
        assert!(source.get_cache_operation(&meta).is_none());
    }

    #[test]
    fn test_method_metadata_debug() {
        let meta = MethodMetadata::new("com.example.Foo", "bar");
        let debug_str = format!("{:?}", meta);
        assert!(debug_str.contains("com.example.Foo"));
        assert!(debug_str.contains("bar"));
    }

    #[test]
    fn test_method_metadata_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        let meta1 = MethodMetadata::new("Foo", "bar");
        let meta2 = MethodMetadata::new("Foo", "baz");
        map.insert(meta1, 1);
        map.insert(meta2, 2);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_method_metadata_clone() {
        let meta = MethodMetadata::new("Foo", "bar");
        let cloned = meta.clone();
        assert_eq!(meta, cloned);
    }
}
