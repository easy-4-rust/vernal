//! 对标 `org.springframework.cache.aspectj` 包。
//!
//! 提供 AspectJ 缓存管理切面的 Rust 等价实现，覆盖：
//! - `AbstractCacheAspect`：缓存切面抽象层
//! - `AnnotationCacheAspect`：`@Cacheable` / `@CachePut` / `@CacheEvict` / `@Caching` 注解驱动
//! - `JCacheCacheAspect`：JSR-107 注解驱动
//! - `AspectJCachingConfiguration`：Spring @Configuration 等价
//! - `AspectJJCacheConfiguration`：JCache @Configuration 等价
//! - `CacheOperation`：缓存操作元数据
//! - `CacheAspectSupport`：缓存核心执行引擎
//! - `AnyThrow`：checked 异常透传辅助

mod abstract_cache_aspect;
mod annotation_cache_aspect;
mod any_throw;
mod aspect_j_caching_configuration;
mod aspect_jj_cache_configuration;
mod cache_aspect_support;
mod cache_operation;
mod cache_operation_source;
mod jcache_cache_aspect;

pub use abstract_cache_aspect::AbstractCacheAspect;
pub use annotation_cache_aspect::AnnotationCacheAspect;
pub use any_throw::AnyThrow;
pub use aspect_j_caching_configuration::AspectJCachingConfiguration;
pub use aspect_jj_cache_configuration::AspectJJCacheConfiguration;
pub use cache_aspect_support::CacheAspectSupport;
pub use cache_operation::CacheOperation;
pub use cache_operation_source::CacheOperationSource;
pub use jcache_cache_aspect::JCacheCacheAspect;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_operation() {
        assert_eq!(CacheOperation::Cacheable, CacheOperation::Cacheable);
        assert_eq!(CacheOperation::CachePut, CacheOperation::CachePut);
        assert_eq!(CacheOperation::CacheEvict, CacheOperation::CacheEvict);
    }

    #[test]
    fn test_cache_operation_source() {
        let source = super::cache_operation_source::AnnotationCacheOperationSource::new();
        let meta = super::cache_operation_source::MethodMetadata::new("Foo", "bar");
        assert!(source.get_cache_operation(&meta).is_none());
    }
}
