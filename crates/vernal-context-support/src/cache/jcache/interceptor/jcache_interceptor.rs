//! JCache 拦截器 — 对标 `JCacheInterceptor`。
use super::jcache_aspect_support::JCacheAspectSupport;
use std::sync::Arc;
use vernal_cache::CacheManager;

pub struct JCacheInterceptor {
    aspect_support: JCacheAspectSupport,
}
impl JCacheInterceptor {
    pub fn new(cache_manager: Arc<dyn CacheManager>) -> Self {
        let mut aspect_support = JCacheAspectSupport::new();
        aspect_support.set_cache_manager(cache_manager);
        Self { aspect_support }
    }
}
