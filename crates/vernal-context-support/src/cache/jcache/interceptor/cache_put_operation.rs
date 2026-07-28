//! @CachePut 操作 — 对标 `CachePutOperation`。
use super::abstract_jcache_operation::AbstractJCacheOperation;
use super::jcache_operation_source::{JCacheOperation, JCacheOperationType};

pub struct CachePutOperation {
    inner: AbstractJCacheOperation,
}
impl CachePutOperation {
    pub fn new(cache_name: String) -> Self {
        Self {
            inner: AbstractJCacheOperation::new(cache_name, JCacheOperationType::CachePut),
        }
    }
}
impl JCacheOperation for CachePutOperation {
    fn cache_name(&self) -> &str {
        self.inner.cache_name()
    }
    fn operation_type(&self) -> JCacheOperationType {
        self.inner.operation_type()
    }
}
