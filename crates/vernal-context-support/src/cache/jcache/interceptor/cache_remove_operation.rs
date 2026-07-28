//! @CacheRemove 操作 — 对标 `CacheRemoveOperation`。
use super::abstract_jcache_operation::AbstractJCacheOperation;
use super::jcache_operation_source::{JCacheOperation, JCacheOperationType};

pub struct CacheRemoveOperation {
    inner: AbstractJCacheOperation,
}
impl CacheRemoveOperation {
    pub fn new(cache_name: String) -> Self {
        Self {
            inner: AbstractJCacheOperation::new(cache_name, JCacheOperationType::CacheRemove),
        }
    }
}
impl JCacheOperation for CacheRemoveOperation {
    fn cache_name(&self) -> &str {
        self.inner.cache_name()
    }
    fn operation_type(&self) -> JCacheOperationType {
        self.inner.operation_type()
    }
}
