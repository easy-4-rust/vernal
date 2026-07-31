//! JCache 拦截器 — 对标 `JCacheInterceptor`。
use super::jcache_aspect_support::JCacheAspectSupport;
use std::sync::Arc;
use vernal_cache::CacheManager;

/// JCache 拦截器。
pub struct JCacheInterceptor {
    // 对标 Spring 的切面支持，暂未读取（Java 镜像脚手架）。
    #[allow(dead_code)]
    aspect_support: JCacheAspectSupport,
}
impl JCacheInterceptor {
    /// 创建拦截器。
    pub fn new(cache_manager: Arc<dyn CacheManager>) -> Self {
        let mut aspect_support = JCacheAspectSupport::new();
        aspect_support.set_cache_manager(cache_manager);
        Self { aspect_support }
    }
}
