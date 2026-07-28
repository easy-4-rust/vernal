//! JCache 操作源 trait — 对标 `org.springframework.cache.jcache.interceptor.JCacheOperationSource`。

use std::any::Any;

/// JCache 操作源 trait。
pub trait JCacheOperationSource: Send + Sync {
    /// 获取方法对应的 JCache 操作。
    fn get_jcache_operation(&self, method: &dyn Any) -> Option<Box<dyn JCacheOperation>>;
    /// 是否支持此方法。
    fn is_candidate(&self, method: &dyn Any) -> bool {
        self.get_jcache_operation(method).is_some()
    }
}

/// JCache 操作 trait。
pub trait JCacheOperation: Send + Sync {
    /// 获取缓存名称。
    fn cache_name(&self) -> &str;
    /// 获取操作类型。
    fn operation_type(&self) -> JCacheOperationType;
}

/// JCache 操作类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JCacheOperationType {
    CacheResult,
    CachePut,
    CacheRemove,
    CacheRemoveAll,
}
