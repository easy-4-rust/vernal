//! 默认操作源 — 对标 `DefaultJCacheOperationSource`。
use super::jcache_operation_source::{JCacheOperation, JCacheOperationSource};
use std::any::Any;

/// 默认操作源。
pub struct DefaultJCacheOperationSource {
    // 对标 Spring 的默认缓存名称，暂未读取（Java 镜像脚手架）。
    #[allow(dead_code)]
    default_cache_name: String,
}
impl DefaultJCacheOperationSource {
    /// 创建默认操作源。
    pub fn new(default_cache_name: String) -> Self {
        Self { default_cache_name }
    }
}
impl JCacheOperationSource for DefaultJCacheOperationSource {
    fn get_jcache_operation(&self, _method: &dyn Any) -> Option<Box<dyn JCacheOperation>> {
        None
    }
}
