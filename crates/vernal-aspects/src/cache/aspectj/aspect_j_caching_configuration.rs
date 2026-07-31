//! 对标 `org.springframework.cache.aspectj.AspectJCachingConfiguration`。
//!
//! 对应 Java：`AspectJCachingConfiguration.java`（`spring-aspects` 模块）
//! 包路径：`org.springframework.cache.aspectj`
//! 核心职责：`@Configuration` 类——注册 `AnnotationCacheAspect` 单例 Bean，启用 `@Cacheable`/`@CachePut`/`@CacheEvict` 声明式缓存管理。

use std::sync::{Arc, RwLock};

use super::annotation_cache_aspect::AnnotationCacheAspect;
use super::cache_operation_source::CacheOperationSource;

/// AspectJ 缓存管理配置。
///
/// 对标 Spring 的 `AspectJCachingConfiguration`。
/// 注册 `AnnotationCacheAspect` 单例 Bean。
pub struct AspectJCachingConfiguration<S: CacheOperationSource + 'static> {
    /// 已注册的缓存切面（懒初始化）。
    aspect: RwLock<Option<Arc<AnnotationCacheAspect<S>>>>,
    /// 缓存操作源。
    cache_operation_source: Arc<S>,
}

impl<S: CacheOperationSource + 'static> AspectJCachingConfiguration<S> {
    /// 创建配置实例。
    pub fn new(cache_operation_source: Arc<S>) -> Self {
        Self {
            aspect: RwLock::new(None),
            cache_operation_source,
        }
    }

    /// 注册缓存切面 Bean。
    ///
    /// 对应 Spring 的 `@Bean(name = CacheManagementConfigUtils.CACHE_ASPECT_BEAN_NAME)`。
    pub fn register(&self) -> Arc<AnnotationCacheAspect<S>> {
        let mut guard = self.aspect.write().unwrap();
        if guard.is_none() {
            *guard = Some(Arc::new(AnnotationCacheAspect::new(
                self.cache_operation_source.clone(),
            )));
        }
        guard.as_ref().unwrap().clone()
    }

    /// 获取已注册的切面。
    pub fn get_aspect(&self) -> Option<Arc<AnnotationCacheAspect<S>>> {
        self.aspect.read().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::aspectj::cache_operation_source::AnnotationCacheOperationSource;

    #[test]
    fn test_configuration_creation() {
        let config = AspectJCachingConfiguration::new(Arc::new(AnnotationCacheOperationSource::new()));
        assert!(config.get_aspect().is_none());
    }

    #[test]
    fn test_configuration_register() {
        let config = AspectJCachingConfiguration::new(Arc::new(AnnotationCacheOperationSource::new()));
        let aspect = config.register();
        assert!(config.get_aspect().is_some());
        // 两次注册返回同一个实例
        let aspect2 = config.register();
        assert!(Arc::ptr_eq(&aspect, &aspect2));
    }

    #[test]
    fn test_configuration_register_multiple() {
        let config = AspectJCachingConfiguration::new(Arc::new(AnnotationCacheOperationSource::new()));
        let _ = config.register();
        let _ = config.register();
        let _ = config.register();
        assert!(config.get_aspect().is_some());
    }
}
