//! JCache AOP 切面支持 — 对标 `JCacheAspectSupport`。
use super::jcache_operation_source::JCacheOperationSource;
use std::sync::Arc;
use vernal_cache::CacheManager;

pub struct JCacheAspectSupport {
    cache_manager: Option<Arc<dyn CacheManager>>,
    operation_source: Option<Box<dyn JCacheOperationSource>>,
}
impl JCacheAspectSupport {
    pub fn new() -> Self {
        Self {
            cache_manager: None,
            operation_source: None,
        }
    }
    pub fn set_cache_manager(&mut self, cm: Arc<dyn CacheManager>) {
        self.cache_manager = Some(cm);
    }
    pub fn set_operation_source(&mut self, os: Box<dyn JCacheOperationSource>) {
        self.operation_source = Some(os);
    }
}
impl Default for JCacheAspectSupport {
    fn default() -> Self {
        Self::new()
    }
}
