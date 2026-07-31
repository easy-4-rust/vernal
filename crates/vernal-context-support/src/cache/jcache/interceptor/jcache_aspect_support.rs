//! JCache AOP 切面支持 — 对标 `JCacheAspectSupport`。
use super::jcache_operation_source::JCacheOperationSource;
use std::sync::Arc;
use vernal_cache::CacheManager;

/// JCache AOP 切面支持。
pub struct JCacheAspectSupport {
    cache_manager: Option<Arc<dyn CacheManager>>,
    operation_source: Option<Box<dyn JCacheOperationSource>>,
}
impl JCacheAspectSupport {
    /// 创建切面支持实例。
    pub fn new() -> Self {
        Self {
            cache_manager: None,
            operation_source: None,
        }
    }
    /// 设置缓存管理器。
    pub fn set_cache_manager(&mut self, cm: Arc<dyn CacheManager>) {
        self.cache_manager = Some(cm);
    }
    /// 设置操作源。
    pub fn set_operation_source(&mut self, os: Box<dyn JCacheOperationSource>) {
        self.operation_source = Some(os);
    }
}
impl Default for JCacheAspectSupport {
    fn default() -> Self {
        Self::new()
    }
}
