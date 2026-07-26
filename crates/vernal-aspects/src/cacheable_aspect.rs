//! 缓存切面（对标 Spring 的 @Cacheable）。
//!
//! 通过 AOP 拦截器实现声明式缓存管理。
//! 与 `vernal-cache` 的 `CacheManager` 配合使用。

use std::sync::Arc;

use vernal_aop::{Interceptor, Invocation, InvocationError, InvocationFuture, Next};

/// 缓存操作类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheOperation {
    /// 读取缓存，如果命中则返回缓存值（@Cacheable）。
    Cacheable,
    /// 更新缓存（@CachePut）。
    CachePut,
    /// 驱逐缓存（@CacheEvict）。
    CacheEvict,
}

/// 缓存配置。
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 缓存名称
    pub cache_name: String,
    /// 缓存操作类型
    pub operation: CacheOperation,
    /// 是否在调用前驱逐缓存（仅 CacheEvict 有效）
    pub before_invocation: bool,
    /// 是否驱逐所有条目（仅 CacheEvict 有效）
    pub all_entries: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            cache_name: String::new(),
            operation: CacheOperation::Cacheable,
            before_invocation: false,
            all_entries: false,
        }
    }
}

/// 缓存切面（对标 Spring 的 CacheInterceptor）。
///
/// 通过 AOP 拦截器实现声明式缓存管理。
///
/// # 使用方式
///
/// ```rust,ignore
/// use vernal_aspects::{CacheableAspect, CacheConfig, CacheOperation};
///
/// let config = CacheConfig {
///     cache_name: "users".to_string(),
///     operation: CacheOperation::Cacheable,
///     ..Default::default()
/// };
///
/// let aspect = CacheableAspect::new(config);
/// ```
pub struct CacheableAspect {
    config: CacheConfig,
}

impl CacheableAspect {
    /// 创建缓存切面。
    #[must_use]
    pub fn new(config: CacheConfig) -> Self {
        Self { config }
    }

    /// 获取缓存配置。
    #[must_use]
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }
}

impl Interceptor for CacheableAspect {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            // TODO: 集成 vernal-cache 的 CacheManager
            // 当前实现：直接执行目标方法，不做缓存管理
            // 完整实现需要：
            // 1. 从 invocation context 获取 CacheManager
            // 2. 根据 config.operation 决定缓存行为
            // 3. Cacheable: 先查缓存，命中则返回，未命中则执行目标并缓存结果
            // 4. CachePut: 执行目标并更新缓存
            // 5. CacheEvict: 驱逐缓存（before/after）

            next.run(invocation).await
        })
    }
}
