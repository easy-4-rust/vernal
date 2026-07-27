//! 对标 `org.springframework.cache.aspectj.AspectJJCacheConfiguration`。
//!
//! JCache 版 `@Configuration`：注册 `JCacheCacheAspect` Bean。

use std::sync::{Arc, RwLock};

use super::jcache_cache_aspect::JCacheCacheAspect;
use super::cache_operation_source::CacheOperationSource;

/// AspectJ JCache 缓存管理配置。
///
/// 对标 Spring 的 `AspectJJCacheConfiguration extends AbstractJCacheConfiguration`。
/// 注册 `JCacheCacheAspect` 单例 Bean。
pub struct AspectJJCacheConfiguration<S: CacheOperationSource + 'static> {
    /// 已注册的 JCache 缓存切面（懒初始化）。
    jcache_aspect: RwLock<Option<Arc<JCacheCacheAspect<S>>>>,
    /// 缓存操作源。
    cache_operation_source: Arc<S>,
}

impl<S: CacheOperationSource + 'static> AspectJJCacheConfiguration<S> {
    /// 创建 JCache 配置实例。
    pub fn new(cache_operation_source: Arc<S>) -> Self {
        Self {
            jcache_aspect: RwLock::new(None),
            cache_operation_source,
        }
    }

    /// 注册 JCache 缓存切面 Bean。
    ///
    /// 对应 Spring 的 `@Bean(name = CacheManagementConfigUtils.JCACHE_ASPECT_BEAN_NAME)`。
    pub fn register_jcache_aspect(&self) -> Arc<JCacheCacheAspect<S>> {
        let mut guard = self.jcache_aspect.write().unwrap();
        if guard.is_none() {
            *guard = Some(Arc::new(JCacheCacheAspect::new(
                self.cache_operation_source.clone(),
            )));
        }
        guard.as_ref().unwrap().clone()
    }

    /// 获取已注册的 JCache 切面。
    pub fn get_jcache_aspect(&self) -> Option<Arc<JCacheCacheAspect<S>>> {
        self.jcache_aspect.read().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::aspectj::cache_operation_source::AnnotationCacheOperationSource;

    #[test]
    fn test_jcache_configuration_creation() {
        let config = AspectJJCacheConfiguration::new(Arc::new(AnnotationCacheOperationSource::new()));
        assert!(config.get_jcache_aspect().is_none());
    }

    #[test]
    fn test_jcache_configuration_register() {
        let config = AspectJJCacheConfiguration::new(Arc::new(AnnotationCacheOperationSource::new()));
        let aspect = config.register_jcache_aspect();
        assert!(config.get_jcache_aspect().is_some());
        let aspect2 = config.register_jcache_aspect();
        assert!(Arc::ptr_eq(&aspect, &aspect2));
    }
}
