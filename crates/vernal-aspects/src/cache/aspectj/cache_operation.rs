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
        assert!(meta.cache_names.is_empty());
        assert!(meta.key.is_none());
        assert!(!meta.before_invocation);
        assert!(!meta.all_entries);
        assert!(!meta.sync);
    }

    #[test]
    fn test_cache_operation_metadata_custom() {
        let meta = CacheOperationMetadata {
            operation: CacheOperation::CacheEvict,
            cache_names: vec![Cow::Borrowed("users")],
            before_invocation: true,
            all_entries: true,
            ..Default::default()
        };
        assert_eq!(meta.operation, CacheOperation::CacheEvict);
        assert_eq!(meta.cache_names.len(), 1);
        assert!(meta.before_invocation);
        assert!(meta.all_entries);
    }

    #[test]
    fn test_cache_operation_is_clone() {
        let op = CacheOperation::Cacheable;
        let op2 = op;
        assert_eq!(op, op2);
    }

    #[test]
    fn test_cache_operation_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<CacheOperation>();
        assert_sync::<CacheOperation>();
    }
}
