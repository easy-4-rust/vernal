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

mod cache_operation;
mod cache_operation_source;
mod cache_aspect_support;
mod abstract_cache_aspect;
mod annotation_cache_aspect;
mod jcache_cache_aspect;
mod aspectj_caching_configuration;
mod aspectj_jcache_configuration;
mod any_throw;

pub use cache_operation::CacheOperation;
pub use cache_operation_source::CacheOperationSource;
pub use cache_aspect_support::CacheAspectSupport;
pub use abstract_cache_aspect::AbstractCacheAspect;
pub use annotation_cache_aspect::AnnotationCacheAspect;
pub use jcache_cache_aspect::JCacheCacheAspect;
pub use aspectj_caching_configuration::AspectJCachingConfiguration;
pub use aspectj_jcache_configuration::AspectJJCacheConfiguration;
pub use any_throw::AnyThrow;
