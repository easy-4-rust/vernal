//! 默认操作源 — 对标 `DefaultJCacheOperationSource`。
use super::jcache_operation_source::{JCacheOperation, JCacheOperationSource};
use std::any::Any;

pub struct DefaultJCacheOperationSource {
    default_cache_name: String,
}
impl DefaultJCacheOperationSource {
    pub fn new(default_cache_name: String) -> Self {
        Self { default_cache_name }
    }
}
impl JCacheOperationSource for DefaultJCacheOperationSource {
    fn get_jcache_operation(&self, _method: &dyn Any) -> Option<Box<dyn JCacheOperation>> {
        None
    }
}
