//! @CacheResult 操作 — 对标 `CacheResultOperation`。
use super::abstract_jcache_operation::AbstractJCacheOperation;
use super::jcache_operation_source::{JCacheOperation, JCacheOperationType};

/// @CacheResult 操作。
pub struct CacheResultOperation {
    inner: AbstractJCacheOperation,
}
impl CacheResultOperation {
    /// 创建 @CacheResult 操作。
    pub fn new(cache_name: String) -> Self {
        Self {
            inner: AbstractJCacheOperation::new(cache_name, JCacheOperationType::CacheResult),
        }
    }
}
impl JCacheOperation for CacheResultOperation {
    fn cache_name(&self) -> &str {
        self.inner.cache_name()
    }
    fn operation_type(&self) -> JCacheOperationType {
        self.inner.operation_type()
    }
}
