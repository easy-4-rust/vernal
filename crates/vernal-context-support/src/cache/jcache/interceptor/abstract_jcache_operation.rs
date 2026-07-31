//! JCache 操作基类 — 对标 `AbstractJCacheOperation`。
use super::jcache_operation_source::{JCacheOperation, JCacheOperationType};

/// JCache 操作基类。
pub struct AbstractJCacheOperation {
    cache_name: String,
    operation_type: JCacheOperationType,
}
impl AbstractJCacheOperation {
    /// 创建操作基类。
    pub fn new(cache_name: String, operation_type: JCacheOperationType) -> Self {
        Self {
            cache_name,
            operation_type,
        }
    }
}
impl JCacheOperation for AbstractJCacheOperation {
    fn cache_name(&self) -> &str {
        &self.cache_name
    }
    fn operation_type(&self) -> JCacheOperationType {
        self.operation_type
    }
}
