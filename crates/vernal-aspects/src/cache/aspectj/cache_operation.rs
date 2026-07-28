//! 对标 `org.springframework.cache.annotation.CacheOperation` 及其子类。
//!
//! 缓存操作元数据：描述一个缓存方法的行为。

use std::borrow::Cow;

/// 缓存操作类型。
///
/// 对标 Spring 的 `Cacheable` / `CachePut` / `CacheEvict` 注解对应的内部操作。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheOperation {
    /// 读取缓存，命中则返回缓存值（`@Cacheable`）。
    Cacheable,
    /// 更新缓存（`@CachePut`）。
    CachePut,
    /// 驱逐缓存（`@CacheEvict`）。
    CacheEvict,
}

/// 缓存操作元数据。
///
/// 对标 Spring 的 `CacheOperation` 抽象类。
/// 描述一个缓存方法的所有属性。
#[derive(Debug, Clone)]
pub struct CacheOperationMetadata {
    /// 操作类型。
    pub operation: CacheOperation,
    /// 缓存名称列表。
    ///
    /// 对应 Spring 的 `CacheOperation#cacheNames`。
    pub cache_names: Vec<Cow<'static, str>>,
    /// 缓存 key 表达式（SpEL）。
    ///
    /// 对应 Spring 的 `CacheOperation#key`。
    pub key: Option<Cow<'static, str>>,
    /// 缓存条件表达式（SpEL）。
    ///
    /// 对应 Spring 的 `CacheOperation#condition`。
    pub condition: Option<Cow<'static, str>>,
    /// 除非表达式（SpEL），为 true 时不缓存结果。
    ///
    /// 对应 Spring 的 `CacheOperation#unless`。
    pub unless: Option<Cow<'static, str>>,
    /// 是否在方法调用前驱逐缓存（仅 `@CacheEvict` 有效）。
    ///
    /// 对应 Spring 的 `CacheEvictOperation#beforeInvocation`。
    pub before_invocation: bool,
    /// 是否驱逐所有条目（仅 `@CacheEvict` 有效）。
    ///
    /// 对应 Spring 的 `CacheEvictOperation#allEntries`。
    pub all_entries: bool,
    /// 是否同步执行（`@Cacheable` 独有）。
    ///
    /// 对应 Spring 的 `CacheableOperation#sync`。
    pub sync: bool,
}

impl Default for CacheOperationMetadata {
    fn default() -> Self {
        Self {
            operation: CacheOperation::Cacheable,
            cache_names: Vec::new(),
            key: None,
            condition: None,
            unless: None,
            before_invocation: false,
            all_entries: false,
            sync: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_operation_variants() {
        assert_eq!(CacheOperation::Cacheable, CacheOperation::Cacheable);
        assert_eq!(CacheOperation::CachePut, CacheOperation::CachePut);
        assert_eq!(CacheOperation::CacheEvict, CacheOperation::CacheEvict);
    }

    #[test]
    fn test_cache_operation_metadata_default() {
        let meta = CacheOperationMetadata::default();
        assert_eq!(meta.operation, CacheOperation::Cacheable);
    }

    #[test]
    fn test_cache_operation_metadata_with_key_and_condition() {
        let meta = CacheOperationMetadata {
            operation: CacheOperation::Cacheable,
            cache_names: vec![Cow::Borrowed("users")],
            key: Some(Cow::Borrowed("#id")),
            condition: Some(Cow::Borrowed("#id > 0")),
            unless: Some(Cow::Borrowed("#result == null")),
            sync: true,
            ..Default::default()
        };
        assert_eq!(meta.key.as_deref(), Some("#id"));
        assert_eq!(meta.condition.as_deref(), Some("#id > 0"));
        assert_eq!(meta.unless.as_deref(), Some("#result == null"));
        assert!(meta.sync);
    }

    #[test]
    fn test_cache_operation_metadata_clone() {
        let meta = CacheOperationMetadata {
            operation: CacheOperation::CachePut,
            cache_names: vec![Cow::Borrowed("users")],
            ..Default::default()
        };
        let cloned = meta.clone();
        assert_eq!(meta.operation, cloned.operation);
        assert_eq!(meta.cache_names, cloned.cache_names);
    }

    #[test]
    fn test_cache_operation_metadata_debug() {
        let meta = CacheOperationMetadata {
            operation: CacheOperation::Cacheable,
            cache_names: vec![Cow::Borrowed("users")],
            ..Default::default()
        };
        let debug_str = format!("{:?}", meta);
        assert!(debug_str.contains("Cacheable"));
    }

    #[test]
    fn test_cache_operation_metadata_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        let meta1 = CacheOperationMetadata {
            operation: CacheOperation::Cacheable,
            cache_names: vec![Cow::Borrowed("users")],
            ..Default::default()
        };
        let meta2 = CacheOperationMetadata {
            operation: CacheOperation::CachePut,
            cache_names: vec![Cow::Borrowed("users")],
            ..Default::default()
        };
        map.insert(format!("{:?}", meta1), 1);
        map.insert(format!("{:?}", meta2), 2);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_cache_operation_metadata_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<CacheOperationMetadata>();
        assert_sync::<CacheOperationMetadata>();
    }

    #[test]
    fn test_cache_operation_metadata_with_all_fields() {
        let meta = CacheOperationMetadata {
            operation: CacheOperation::Cacheable,
            cache_names: vec![Cow::Borrowed("users"), Cow::Borrowed("orders")],
            key: Some(Cow::Borrowed("#id")),
            condition: Some(Cow::Borrowed("#id > 0")),
            unless: Some(Cow::Borrowed("#result == null")),
            before_invocation: true,
            all_entries: true,
            sync: true,
        };

        assert_eq!(meta.operation, CacheOperation::Cacheable);
        assert_eq!(meta.cache_names.len(), 2);
        assert_eq!(meta.key.as_deref(), Some("#id"));
        assert_eq!(meta.condition.as_deref(), Some("#id > 0"));
        assert_eq!(meta.unless.as_deref(), Some("#result == null"));
        assert!(meta.before_invocation);
        assert!(meta.all_entries);
        assert!(meta.sync);
    }

    #[test]
    fn test_cache_operation_metadata_with_no_fields() {
        let meta = CacheOperationMetadata::default();
        assert_eq!(meta.operation, CacheOperation::Cacheable);
        assert!(meta.cache_names.is_empty());
        assert!(meta.key.is_none());
        assert!(meta.condition.is_none());
        assert!(meta.unless.is_none());
        assert!(!meta.before_invocation);
        assert!(!meta.all_entries);
        assert!(!meta.sync);
    }

    #[test]
    fn test_cache_operation_debug() {
        let op = CacheOperation::Cacheable;
        let debug_str = format!("{:?}", op);
        assert!(debug_str.contains("Cacheable"));
    }

    #[test]
    fn test_cache_operation_clone() {
        let op = CacheOperation::Cacheable;
        let cloned = op;
        assert_eq!(op, cloned);
    }

    #[test]
    fn test_cache_operation_hash() {
        let mut map = std::collections::HashMap::new();
        map.insert(CacheOperation::Cacheable, 1);
        map.insert(CacheOperation::CachePut, 2);
        map.insert(CacheOperation::CacheEvict, 3);
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn test_cache_operation_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<CacheOperation>();
        assert_sync::<CacheOperation>();
    }

    #[test]
    fn test_cache_operation_metadata_ne() {
        let meta1 = CacheOperationMetadata::default();
        let meta2 = CacheOperationMetadata {
            operation: CacheOperation::CachePut,
            ..Default::default()
        };
        assert_ne!(meta1.operation, meta2.operation);
    }
}
