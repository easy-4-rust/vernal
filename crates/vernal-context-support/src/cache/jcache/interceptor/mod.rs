//! JCache 拦截器 — 对标 `org.springframework.cache.jcache.interceptor`。
//!
//! 精简了 Spring 的 19 个拦截器类，保留核心 trait 和 11 个实现文件。

mod abstract_jcache_operation;
mod annotation_jcache_operation_source;
mod cache_put_operation;
mod cache_remove_all_operation;
mod cache_remove_operation;
mod cache_resolver_adapter;
mod cache_result_operation;
mod default_jcache_operation_source;
mod jcache_aspect_support;
mod jcache_interceptor;
mod jcache_operation_source;
mod simple_exception_cache_resolver;

pub use abstract_jcache_operation::AbstractJCacheOperation;
pub use annotation_jcache_operation_source::AnnotationJCacheOperationSource;
pub use cache_put_operation::CachePutOperation;
pub use cache_remove_all_operation::CacheRemoveAllOperation;
pub use cache_remove_operation::CacheRemoveOperation;
pub use cache_resolver_adapter::CacheResolverAdapter;
pub use cache_result_operation::CacheResultOperation;
pub use default_jcache_operation_source::DefaultJCacheOperationSource;
pub use jcache_aspect_support::JCacheAspectSupport;
pub use jcache_interceptor::JCacheInterceptor;
pub use jcache_operation_source::JCacheOperationSource;
pub use simple_exception_cache_resolver::SimpleExceptionCacheResolver;
