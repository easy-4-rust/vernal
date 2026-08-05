//! 对标 `org.springframework.cache.aspectj.JCacheCacheAspect`。
//!
//! 基于 JSR-107（`javax.cache` / `jakarta.cache`）注解驱动的缓存切面。

use std::sync::Arc;

use super::cache_aspect_support::{CacheOperationInvoker, CacheResult};
use super::cache_operation_source::{CacheOperationSource, MethodMetadata};

/// 基于 JSR-107 注解的缓存切面。
///
/// 对标 Spring 的 `JCacheCacheAspect`。
/// 使用 JSR-107 标准注解：`@CacheResult` / `@CachePut` / `@CacheRemove` / `@CacheRemoveAll`。
///
/// # Pointcut 定义
///
/// ```text
/// (executionOfCacheResultMethod()
///     || executionOfCachePutMethod()
///     || executionOfCacheRemoveMethod()
///     || executionOfCacheRemoveAllMethod())
///     && this(cachedObject)
/// ```
pub struct JCacheCacheAspect<S: CacheOperationSource> {
    /// 缓存操作源（Java 镜像字段，当前阶段暂未直接读取）。
    #[allow(dead_code)]
    cache_operation_source: Arc<S>,
}

impl<S: CacheOperationSource> JCacheCacheAspect<S> {
    /// 创建 JCache 缓存切面。
    pub fn new(cache_operation_source: Arc<S>) -> Self {
        Self {
            cache_operation_source,
        }
    }

    /// 匹配 `@CacheResult` 方法。
    pub fn matches_cache_result_method(&self, method_has_cache_result: bool) -> bool {
        method_has_cache_result
    }

    /// 匹配 `@CachePut` 方法。
    pub fn matches_jcache_put_method(&self, method_has_jcache_put: bool) -> bool {
        method_has_jcache_put
    }

    /// 匹配 `@CacheRemove` 方法。
    pub fn matches_cache_remove_method(&self, method_has_cache_remove: bool) -> bool {
        method_has_cache_remove
    }

    /// 匹配 `@CacheRemoveAll` 方法。
    pub fn matches_cache_remove_all_method(&self, method_has_cache_remove_all: bool) -> bool {
        method_has_cache_remove_all
    }

    /// 组合 pointcut：`cacheMethodExecution(Object cachedObject)`。
    pub fn cache_method_execution(
        &self,
        method_has_cache_result: bool,
        method_has_jcache_put: bool,
        method_has_cache_remove: bool,
        method_has_cache_remove_all: bool,
        this_matches: bool,
    ) -> bool {
        if !this_matches {
            return false;
        }
        self.matches_cache_result_method(method_has_cache_result)
            || self.matches_jcache_put_method(method_has_jcache_put)
            || self.matches_cache_remove_method(method_has_cache_remove)
            || self.matches_cache_remove_all_method(method_has_cache_remove_all)
    }

    /// 执行缓存操作。
    pub fn execute<F>(
        &self,
        _method: &MethodMetadata,
        _target_type_name: &str,
        _invoker: &dyn CacheOperationInvoker,
        callback: F,
    ) -> CacheResult
    where
        F: FnOnce() -> Result<
            Box<dyn std::any::Any + Send + Sync>,
            Box<dyn std::any::Any + Send + Sync>,
        >,
    {
        // 实际实现需要调用 CacheAspectSupport
        match callback() {
            Ok(result) => CacheResult::Hit(result),
            Err(err) => CacheResult::Error(format!("Execution failed: {:?}", err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::aspectj::cache_operation_source::AnnotationCacheOperationSource;

    struct MockInvoker;
    impl CacheOperationInvoker for MockInvoker {
        fn invoke(
            &self,
        ) -> Result<Box<dyn std::any::Any + Send + Sync>, Box<dyn std::any::Any + Send + Sync>>
        {
            Ok(Box::new(42) as Box<dyn std::any::Any + Send + Sync>)
        }
    }

    #[test]
    fn test_jcache_cache_aspect_creation() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = JCacheCacheAspect::new(source);
        assert!(aspect.matches_cache_result_method(true));
        assert!(!aspect.matches_cache_result_method(false));
    }

    #[test]
    fn test_matches_jcache_put_method() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = JCacheCacheAspect::new(source);
        assert!(aspect.matches_jcache_put_method(true));
        assert!(!aspect.matches_jcache_put_method(false));
    }

    #[test]
    fn test_matches_cache_remove_method() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = JCacheCacheAspect::new(source);
        assert!(aspect.matches_cache_remove_method(true));
        assert!(!aspect.matches_cache_remove_method(false));
    }

    #[test]
    fn test_matches_cache_remove_all_method() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = JCacheCacheAspect::new(source);
        assert!(aspect.matches_cache_remove_all_method(true));
        assert!(!aspect.matches_cache_remove_all_method(false));
    }

    #[test]
    fn test_cache_method_execution_combination() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = JCacheCacheAspect::new(source);

        assert!(!aspect.cache_method_execution(true, false, false, false, false));
        assert!(aspect.cache_method_execution(true, false, false, false, true));
        assert!(aspect.cache_method_execution(false, true, false, false, true));
        assert!(aspect.cache_method_execution(false, false, true, false, true));
        assert!(aspect.cache_method_execution(false, false, false, true, true));
        assert!(!aspect.cache_method_execution(false, false, false, false, true));
    }

    #[test]
    fn test_execute_success() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = JCacheCacheAspect::new(source);
        let invoker = MockInvoker;
        let method = super::super::cache_operation_source::MethodMetadata::new("Foo", "bar");

        let result = aspect.execute(&method, "Foo", &invoker, || {
            Ok(Box::new(42) as Box<dyn std::any::Any + Send + Sync>)
        });

        match result {
            CacheResult::Hit(val) => {
                assert_eq!(val.downcast_ref::<i32>().unwrap(), &42);
            }
            _ => panic!("Expected Hit result"),
        }
    }

    #[test]
    fn test_execute_error() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = JCacheCacheAspect::new(source);
        let invoker = MockInvoker;
        let method = super::super::cache_operation_source::MethodMetadata::new("Foo", "bar");

        let result = aspect.execute(&method, "Foo", &invoker, || {
            Err(Box::new("error") as Box<dyn std::any::Any + Send + Sync>)
        });

        match result {
            CacheResult::Error(msg) => {
                assert!(msg.contains("Execution failed"));
            }
            _ => panic!("Expected Error result"),
        }
    }
}
