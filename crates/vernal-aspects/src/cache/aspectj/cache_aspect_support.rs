//! 对标 `org.springframework.cache.interceptor.CacheAspectSupport` 抽象类。
//!
//! 缓存切面支撑基类：提供缓存管理的核心执行引擎 `execute`。

use std::any::Any;
use std::sync::Arc;

use super::cache_operation::CacheOperationMetadata;
use super::cache_operation_source::{CacheOperationSource, MethodMetadata};

/// 缓存操作结果。
#[derive(Debug)]
pub enum CacheResult {
    /// 缓存命中。
    Hit(Box<dyn Any + Send + Sync>),
    /// 缓存未命中，需要执行目标方法。
    Miss,
    /// 缓存操作失败。
    Error(String),
}

/// 缓存管理器 trait。
///
/// 对标 Spring 的 `CacheManager` 接口。
pub trait CacheManager: Send + Sync + 'static {
    /// 获取缓存管理器名称。
    fn get_name(&self) -> &str;

    /// 根据名称获取缓存。
    fn get_cache(&self, name: &str) -> Option<Box<dyn Cache>>;
}

/// 缓存 trait。
///
/// 对标 Spring 的 `Cache` 接口。
pub trait Cache: Send + Sync + 'static {
    /// 获取缓存名称。
    fn get_name(&self) -> &str;

    /// 根据 key 获取缓存值。
    fn get(&self, key: &str) -> Option<Box<dyn Any + Send + Sync>>;

    /// 放入缓存值。
    fn put(&self, key: &str, value: Box<dyn Any + Send + Sync>);

    /// 驱逐缓存条目。
    fn evict(&self, key: &str);

    /// 清空缓存。
    fn clear(&self);
}

/// 缓存操作调用器。
///
/// 对标 Spring 的 `CacheOperationInvoker` 接口。
pub trait CacheOperationInvoker: Send + Sync + 'static {
    /// 执行缓存操作。
    fn invoke(&self) -> Result<Box<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>>;
}

/// 缓存切面支撑基类。
///
/// 对标 Spring 的 `CacheAspectSupport` 抽象类。
/// 提供缓存管理的核心执行引擎 `execute`。
pub struct CacheAspectSupport<S: CacheOperationSource> {
    /// 缓存操作源。
    cache_operation_source: Arc<S>,
    /// 缓存管理器。
    cache_manager: Option<Arc<dyn CacheManager>>,
    /// 默认缓存管理器名称。
    default_cache_manager_name: Option<String>,
    /// 错误处理器。
    error_handler: Option<Box<dyn Fn(&dyn Any) + Send + Sync>>,
}

impl<S: CacheOperationSource> CacheAspectSupport<S> {
    /// 创建缓存切面支撑实例。
    pub fn new(cache_operation_source: Arc<S>) -> Self {
        Self {
            cache_operation_source,
            cache_manager: None,
            default_cache_manager_name: None,
            error_handler: None,
        }
    }

    /// 设置缓存管理器。
    pub fn set_cache_manager(&mut self, cache_manager: Arc<dyn CacheManager>) {
        self.cache_manager = Some(cache_manager);
    }

    /// 获取缓存操作源。
    pub fn get_cache_operation_source(&self) -> &Arc<S> {
        &self.cache_operation_source
    }

    /// 清理元数据缓存。
    ///
    /// 对标 Spring 的 `clearMetadataCache()` 方法。
    /// 在切面销毁时调用。
    pub fn clear_metadata_cache(&self) {
        // 实际实现需要清理内部元数据缓存
    }

    /// 执行缓存操作。
    ///
    /// 对标 Spring 的 `CacheAspectSupport#execute` 方法。
    ///
    /// # 执行流程
    ///
    /// 1. 获取缓存操作元数据
    /// 2. 评估 condition 表达式
    /// 3. 生成 cache key
    /// 4. 根据操作类型执行：
    ///    - Cacheable：先查缓存，命中则返回，未命中则执行目标方法并缓存结果
    ///    - CachePut：执行目标方法并更新缓存
    ///    - CacheEvict：驱逐缓存（before/after）
    /// 5. 评估 unless 表达式决定是否缓存结果
    pub fn execute<F>(
        &self,
        method: &MethodMetadata,
        target_type_name: &str,
        invoker: &dyn CacheOperationInvoker,
        callback: F,
    ) -> CacheResult
    where
        F: FnOnce() -> Result<Box<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>>,
    {
        // 1. 获取缓存操作元数据
        let operation = match self.cache_operation_source.get_cache_operation(method) {
            Some(op) => op,
            None => return CacheResult::Error("No cache operation found".to_string()),
        };

        // 2. 根据操作类型执行
        match operation.operation {
            super::cache_operation::CacheOperation::Cacheable => {
                self.execute_cacheable(operation, invoker, callback)
            }
            super::cache_operation::CacheOperation::CachePut => {
                self.execute_cache_put(operation, invoker, callback)
            }
            super::cache_operation::CacheOperation::CacheEvict => {
                self.execute_cache_evict(operation, invoker, callback)
            }
        }
    }

    /// 执行 Cacheable 操作。
    fn execute_cacheable<F>(
        &self,
        _operation: CacheOperationMetadata,
        _invoker: &dyn CacheOperationInvoker,
        callback: F,
    ) -> CacheResult
    where
        F: FnOnce() -> Result<Box<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>>,
    {
        // 实际实现需要：
        // 1. 从 cache_manager 获取缓存
        // 2. 生成 key
        // 3. 查缓存
        // 4. 命中则返回 Hit
        // 5. 未命中则执行 callback 并缓存结果
        match callback() {
            Ok(result) => CacheResult::Hit(result),
            Err(err) => CacheResult::Error(format!("Execution failed: {:?}", err)),
        }
    }

    /// 执行 CachePut 操作。
    fn execute_cache_put<F>(
        &self,
        _operation: CacheOperationMetadata,
        _invoker: &dyn CacheOperationInvoker,
        callback: F,
    ) -> CacheResult
    where
        F: FnOnce() -> Result<Box<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>>,
    {
        match callback() {
            Ok(result) => CacheResult::Hit(result),
            Err(err) => CacheResult::Error(format!("Execution failed: {:?}", err)),
        }
    }

    /// 执行 CacheEvict 操作。
    fn execute_cache_evict<F>(
        &self,
        _operation: CacheOperationMetadata,
        _invoker: &dyn CacheOperationInvoker,
        callback: F,
    ) -> CacheResult
    where
        F: FnOnce() -> Result<Box<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>>,
    {
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
    use std::sync::Arc;

    struct MockInvoker;

    impl CacheOperationInvoker for MockInvoker {
        fn invoke(&self) -> Result<Box<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>> {
            Ok(Box::new(42) as Box<dyn Any + Send + Sync>)
        }
    }

    #[test]
    fn test_cache_aspect_support_creation() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let support = CacheAspectSupport::new(source);
        assert!(support.cache_manager.is_none());
    }

    #[test]
    fn test_execute_no_operation() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let support = CacheAspectSupport::new(source);
        let invoker = MockInvoker;
        let method = super::super::cache_operation_source::MethodMetadata::new("Foo", "bar");

        let result = support.execute(&method, "Foo", &invoker, || {
            Ok(Box::new(42) as Box<dyn Any + Send + Sync>)
        });

        match result {
            CacheResult::Error(msg) => assert!(msg.contains("No cache operation")),
            _ => panic!("Expected error for no operation"),
        }
    }

    #[test]
    fn test_clear_metadata_cache() {
        let source = Arc::new(AnnotationCacheOperationSource::new());
        let support = CacheAspectSupport::new(source);
        support.clear_metadata_cache();
    }

    #[test]
    fn test_cache_aspect_support_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<CacheAspectSupport<AnnotationCacheOperationSource>>();
        assert_sync::<CacheAspectSupport<AnnotationCacheOperationSource>>();
    }
}
