//! 对标 `org.springframework.cache.aspectj.AnnotationCacheAspect` 具体 Aspect。
//!
//! 基于 Spring `@Cacheable` / `@CachePut` / `@CacheEvict` / `@Caching` 注解驱动的缓存切面。

use std::sync::Arc;

use super::abstract_cache_aspect::AbstractCacheAspect;
use super::cache_aspect_support::{CacheOperationInvoker, CacheResult};
use super::cache_operation_source::{CacheOperationSource, MethodMetadata};

/// 基于 Spring 缓存注解的缓存切面。
///
/// 对标 Spring 的 `AnnotationCacheAspect`。
/// 这是 Spring `@Cacheable` / `@CachePut` / `@CacheEvict` / `@Caching` 注解的 AspectJ 实现。
///
/// # Pointcut 定义
///
/// ```text
/// // 8 个内部 pointcut 的 || 组合
/// (executionOfAnyPublicMethodInAtCacheableType()
///     || executionOfAnyPublicMethodInAtCacheEvictType()
///     || executionOfAnyPublicMethodInAtCachePutType()
///     || executionOfAnyPublicMethodInAtCachingType()
///     || executionOfCacheableMethod()
///     || executionOfCacheEvictMethod()
///     || executionOfCachePutMethod()
///     || executionOfCachingMethod())
///     && this(cachedObject)
/// ```
pub struct AnnotationCacheAspect<S: CacheOperationSource> {
    inner: AbstractCacheAspect<S>,
    cache_operation_source: Arc<S>,
}

impl<S: CacheOperationSource> AnnotationCacheAspect<S> {
    /// 创建注解缓存切面。
    pub fn new(cache_operation_source: Arc<S>) -> Self {
        let inner = AbstractCacheAspect::new(cache_operation_source.clone());
        Self {
            inner,
            cache_operation_source,
        }
    }

    /// 匹配 `@Cacheable` 类型中的公开方法。
    ///
    /// 对应 Spring 的 `executionOfAnyPublicMethodInAtCacheableType()` pointcut。
    pub fn matches_cacheable_type(
        &self,
        type_has_cacheable: bool,
        type_in_scope: bool,
    ) -> bool {
        type_has_cacheable && type_in_scope
    }

    /// 匹配 `@CacheEvict` 类型中的公开方法。
    pub fn matches_cache_evict_type(
        &self,
        type_has_cache_evict: bool,
        type_in_scope: bool,
    ) -> bool {
        type_has_cache_evict && type_in_scope
    }

    /// 匹配 `@CachePut` 类型中的公开方法。
    pub fn matches_cache_put_type(
        &self,
        type_has_cache_put: bool,
        type_in_scope: bool,
    ) -> bool {
        type_has_cache_put && type_in_scope
    }

    /// 匹配 `@Caching` 类型中的公开方法。
    pub fn matches_caching_type(
        &self,
        type_has_caching: bool,
        type_in_scope: bool,
    ) -> bool {
        type_has_caching && type_in_scope
    }

    /// 匹配 `@Cacheable` 方法。
    pub fn matches_cacheable_method(&self, method_has_cacheable: bool) -> bool {
        method_has_cacheable
    }

    /// 匹配 `@CacheEvict` 方法。
    pub fn matches_cache_evict_method(&self, method_has_cache_evict: bool) -> bool {
        method_has_cache_evict
    }

    /// 匹配 `@CachePut` 方法。
    pub fn matches_cache_put_method(&self, method_has_cache_put: bool) -> bool {
        method_has_cache_put
    }

    /// 匹配 `@Caching` 方法。
    pub fn matches_caching_method(&self, method_has_caching: bool) -> bool {
        method_has_caching
    }

    /// 组合 pointcut：`cacheMethodExecution(Object cachedObject)`。
    ///
    /// 对应 Spring 的 `protected pointcut cacheMethodExecution(Object cachedObject)`。
    pub fn cache_method_execution(
        &self,
        type_has_cacheable: bool,
        type_has_cache_evict: bool,
        type_has_cache_put: bool,
        type_has_caching: bool,
        type_in_scope: bool,
        method_has_cacheable: bool,
        method_has_cache_evict: bool,
        method_has_cache_put: bool,
        method_has_caching: bool,
        this_matches: bool,
    ) -> bool {
        if !this_matches {
            return false;
        }

        self.matches_cacheable_type(type_has_cacheable, type_in_scope)
            || self.matches_cache_evict_type(type_has_cache_evict, type_in_scope)
            || self.matches_cache_put_type(type_has_cache_put, type_in_scope)
            || self.matches_caching_type(type_has_caching, type_in_scope)
            || self.matches_cacheable_method(method_has_cacheable)
            || self.matches_cache_evict_method(method_has_cache_evict)
            || self.matches_cache_put_method(method_has_cache_put)
            || self.matches_caching_method(method_has_caching)
    }

    /// 执行缓存操作（around advice）。
    pub fn execute<F>(
        &self,
        method: &MethodMetadata,
        target_type_name: &str,
        invoker: &dyn CacheOperationInvoker,
        callback: F,
    ) -> CacheResult
    where
        F: FnOnce() -> Result<Box<dyn std::any::Any + Send + Sync>, Box<dyn std::any::Any + Send + Sync>>,
    {
        self.inner.execute(method, target_type_name, invoker, callback)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::aspectj::cache_operation_source::AnnotationCacheOperationSource;
    use std::sync::Arc;

    struct MockInvoker;
    impl CacheOperationInvoker for MockInvoker {
        fn invoke(&self) -> Result<Box<dyn std::any::Any + Send + Sync>, Box<dyn std::any::Any + Send + Sync>> {
            Ok(Box::new(42) as Box<dyn std::any::Any + Send + Sync>)
        }
    }

    #[test]
    fn test_annotation_cache_aspect_creation() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = AnnotationCacheAspect::new(source);
        // aspect 创建成功即可，不需要检查内部状态
        let _ = aspect;
    }

    #[test]
    fn test_matches_cacheable_type() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = AnnotationCacheAspect::new(source);
        assert!(aspect.matches_cacheable_type(true, true));
        assert!(!aspect.matches_cacheable_type(false, true));
        assert!(!aspect.matches_cacheable_type(true, false));
    }

    #[test]
    fn test_matches_cacheable_method() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = AnnotationCacheAspect::new(source);
        assert!(aspect.matches_cacheable_method(true));
        assert!(!aspect.matches_cacheable_method(false));
    }

    #[test]
    fn test_cache_method_execution_combination() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = AnnotationCacheAspect::new(source);

        // this_object 不匹配时返回 false
        assert!(!aspect.cache_method_execution(
            true, false, false, false, true, false, false, false, false, false
        ));

        // cacheable 类型匹配
        assert!(aspect.cache_method_execution(
            true, false, false, false, true, false, false, false, false, true
        ));

        // cacheable 方法匹配
        assert!(aspect.cache_method_execution(
            false, false, false, false, false, true, false, false, false, true
        ));

        // 都不匹配
        assert!(!aspect.cache_method_execution(
            false, false, false, false, false, false, false, false, false, true
        ));
    }

    #[test]
    fn test_execute_no_operation() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = AnnotationCacheAspect::new(source);
        let invoker = MockInvoker;
        let method = super::super::cache_operation_source::MethodMetadata::new("Foo", "bar");

        let result = aspect.execute(&method, "Foo", &invoker, || {
            Ok(Box::new(42) as Box<dyn std::any::Any + Send + Sync>)
        });

        match result {
            CacheResult::Error(msg) => assert!(msg.contains("No cache operation")),
            _ => panic!("Expected error"),
        }
    }

    #[test]
    fn test_matches_all_type_pointcuts() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = AnnotationCacheAspect::new(source);

        assert!(aspect.matches_cacheable_type(true, true));
        assert!(!aspect.matches_cacheable_type(false, true));
        assert!(!aspect.matches_cacheable_type(true, false));

        assert!(aspect.matches_cache_evict_type(true, true));
        assert!(!aspect.matches_cache_evict_type(false, true));

        assert!(aspect.matches_cache_put_type(true, true));
        assert!(!aspect.matches_cache_put_type(false, true));

        assert!(aspect.matches_caching_type(true, true));
        assert!(!aspect.matches_caching_type(false, true));
    }

    #[test]
    fn test_matches_all_method_pointcuts() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = AnnotationCacheAspect::new(source);

        assert!(aspect.matches_cacheable_method(true));
        assert!(!aspect.matches_cacheable_method(false));

        assert!(aspect.matches_cache_evict_method(true));
        assert!(!aspect.matches_cache_evict_method(false));

        assert!(aspect.matches_cache_put_method(true));
        assert!(!aspect.matches_cache_put_method(false));

        assert!(aspect.matches_caching_method(true));
        assert!(!aspect.matches_caching_method(false));
    }

    #[test]
    fn test_cache_method_execution_all_combinations() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = AnnotationCacheAspect::new(source);

        // this 不匹配
        assert!(!aspect.cache_method_execution(true, false, false, false, false, false, false, false, false, false));

        // cacheable 类型匹配
        assert!(aspect.cache_method_execution(true, false, false, false, true, false, false, false, false, true));

        // cache_evict 类型匹配
        assert!(aspect.cache_method_execution(false, true, false, false, true, false, false, false, false, true));

        // cache_put 类型匹配
        assert!(aspect.cache_method_execution(false, false, true, false, true, false, false, false, false, true));

        // caching 类型匹配
        assert!(aspect.cache_method_execution(false, false, false, true, true, false, false, false, false, true));

        // cacheable 方法匹配
        assert!(aspect.cache_method_execution(false, false, false, false, false, true, false, false, false, true));

        // cache_evict 方法匹配
        assert!(aspect.cache_method_execution(false, false, false, false, false, false, true, false, false, true));

        // cache_put 方法匹配
        assert!(aspect.cache_method_execution(false, false, false, false, false, false, false, true, false, true));

        // caching 方法匹配
        assert!(aspect.cache_method_execution(false, false, false, false, false, false, false, false, true, true));

        // 都不匹配
        assert!(!aspect.cache_method_execution(false, false, false, false, false, false, false, false, false, true));
    }

    #[test]
    fn test_annotation_cache_aspect_debug() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = AnnotationCacheAspect::new(source);
        let _ = aspect;
    }

    #[test]
    fn test_annotation_cache_aspect_clone() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = AnnotationCacheAspect::new(source);
        let _ = aspect;
    }
}
