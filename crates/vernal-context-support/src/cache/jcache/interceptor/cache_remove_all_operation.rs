//! @CacheRemoveAll 操作 — 对标 `CacheRemoveAllOperation`。
use super::abstract_jcache_operation::AbstractJCacheOperation;
use super::jcache_operation_source::{JCacheOperation, JCacheOperationType};

pub struct CacheRemoveAllOperation {
    inner: AbstractJCacheOperation,
}
impl CacheRemoveAllOperation {
    pub fn new(cache_name: String) -> Self {
        Self {
            inner: AbstractJCacheOperation::new(cache_name, JCacheOperationType::CacheRemoveAll),
        }
    }
}
impl JCacheOperation for CacheRemoveAllOperation {
    fn cache_name(&self) -> &str {
        self.inner.cache_name()
    }
    fn operation_type(&self) -> JCacheOperationType {
        self.inner.operation_type()
    }
}
