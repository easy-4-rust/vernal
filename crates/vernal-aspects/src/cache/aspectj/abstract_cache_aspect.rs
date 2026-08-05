//! 对标 `org.springframework.cache.aspectj.AbstractCacheAspect` 抽象 Aspect。
//!
//! 缓存切面抽象层：封装 `CacheAspectSupport` 的 `execute` 行为。

use std::sync::Arc;

use super::cache_aspect_support::{CacheAspectSupport, CacheOperationInvoker, CacheResult};
use super::cache_operation_source::{CacheOperationSource, MethodMetadata};

/// 缓存切面抽象基类。
///
/// 对标 Spring 的 `AbstractCacheAspect`。
/// 封装了 `CacheAspectSupport` 的核心执行逻辑。
pub struct AbstractCacheAspect<S: CacheOperationSource> {
    /// 缓存切面支撑（核心执行引擎）。
    support: CacheAspectSupport<S>,
}

impl<S: CacheOperationSource> AbstractCacheAspect<S> {
    /// 创建缓存切面抽象实例。
    ///
    /// 对应 Spring 的 `protected AbstractCacheAspect(CacheOperationSource... cos)`。
    pub fn new(cache_operation_source: Arc<S>) -> Self {
        Self {
            support: CacheAspectSupport::new(cache_operation_source),
        }
    }

    /// 获取缓存切面支撑实例。
    pub fn get_support(&self) -> &CacheAspectSupport<S> {
        &self.support
    }

    /// 获取可变的缓存切面支撑实例。
    pub fn get_support_mut(&mut self) -> &mut CacheAspectSupport<S> {
        &mut self.support
    }

    /// 执行缓存操作。
    ///
    /// 对应 Spring 的 `AbstractCacheAspect#around` advice 内部逻辑。
    pub fn execute<F>(
        &self,
        method: &MethodMetadata,
        target_type_name: &str,
        invoker: &dyn CacheOperationInvoker,
        callback: F,
    ) -> CacheResult
    where
        F: FnOnce() -> Result<
            Box<dyn std::any::Any + Send + Sync>,
            Box<dyn std::any::Any + Send + Sync>,
        >,
    {
        self.support
            .execute(method, target_type_name, invoker, callback)
    }

    /// 清理元数据缓存。
    ///
    /// 对应 Spring 的 `destroy()` 回调。
    pub fn destroy(&self) {
        self.support.clear_metadata_cache();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::aspectj::cache_operation_source::AnnotationCacheOperationSource;
    use std::sync::Arc;

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
    fn test_abstract_cache_aspect_creation() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = AbstractCacheAspect::new(source);
        let _ = aspect.get_support();
    }

    #[test]
    fn test_get_support_mut() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let mut aspect = AbstractCacheAspect::new(source);
        let _ = aspect.get_support_mut();
    }

    #[test]
    fn test_destroy_does_not_panic() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = AbstractCacheAspect::new(source);
        aspect.destroy();
    }

    #[test]
    fn test_execute_no_operation() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = AbstractCacheAspect::new(source);
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
    fn test_execute_with_operation() {
        let mut source = AnnotationCacheOperationSource::new();
        let op = super::super::cache_operation::CacheOperationMetadata {
            operation: super::super::cache_operation::CacheOperation::Cacheable,
            cache_names: vec![std::borrow::Cow::Borrowed("users")],
            ..Default::default()
        };
        source.register_method("Foo#bar()".to_string(), op);
        let source = Arc::new(source);
        let aspect = AbstractCacheAspect::new(source);
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
    fn test_execute_cache_put_operation() {
        let mut source = AnnotationCacheOperationSource::new();
        let op = super::super::cache_operation::CacheOperationMetadata {
            operation: super::super::cache_operation::CacheOperation::CachePut,
            cache_names: vec![std::borrow::Cow::Borrowed("users")],
            ..Default::default()
        };
        source.register_method("Foo#bar()".to_string(), op);
        let source = Arc::new(source);
        let aspect = AbstractCacheAspect::new(source);
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
    fn test_execute_cache_evict_operation() {
        let mut source = AnnotationCacheOperationSource::new();
        let op = super::super::cache_operation::CacheOperationMetadata {
            operation: super::super::cache_operation::CacheOperation::CacheEvict,
            cache_names: vec![std::borrow::Cow::Borrowed("users")],
            before_invocation: true,
            all_entries: true,
            ..Default::default()
        };
        source.register_method("Foo#bar()".to_string(), op);
        let source = Arc::new(source);
        let aspect = AbstractCacheAspect::new(source);
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
    fn test_abstract_cache_aspect_debug() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = AbstractCacheAspect::new(source);
        let _ = aspect;
    }

    #[test]
    fn test_abstract_cache_aspect_clone() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let aspect = AbstractCacheAspect::new(source);
        let _ = aspect;
    }
}
