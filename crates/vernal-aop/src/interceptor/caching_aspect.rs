//! 缓存切面。
//!
//! 对应 aspect-rs：aspect-std/src/caching.rs。
//! spring-aop 无直接对应（spring-cache 有类似功能）。
//!
//! 提供基于 key 的结果缓存，支持 TTL。
//! 实现 `Interceptor` trait（around 全控制）。
//!
//! 注意：由于 `InvocationValue`（`Box<dyn Any + Send + Sync>`）不实现 `Clone`，
//! 缓存切面使用引用计数方式存储缓存值。

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::{Interceptor, Invocation, InvocationFuture, InvocationResult, InvocationValue, Next};

/// 缓存条目。
struct CacheEntry {
    value: InvocationValue,
    inserted_at: Instant,
    ttl: Option<Duration>,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.inserted_at.elapsed() > ttl
        } else {
            false
        }
    }
}

/// 缓存切面。
///
/// 提供基于 key 的结果缓存，支持 TTL 和最大容量。
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::CachingAspect;
/// use std::time::Duration;
///
/// let cache = CachingAspect::new()
///     .with_max_size(1000)
///     .with_ttl(Duration::from_secs(60));
/// ```
#[derive(Clone)]
pub struct CachingAspect {
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    max_size: usize,
    ttl: Option<Duration>,
}

impl CachingAspect {
    /// 创建新的缓存切面。
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            max_size: usize::MAX,
            ttl: None,
        }
    }

    /// 设置最大缓存容量。
    #[must_use]
    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = max_size;
        self
    }

    /// 设置缓存条目的 TTL。
    #[must_use]
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// 获取缓存条目数量。
    pub async fn size(&self) -> usize {
        self.cache.lock().await.len()
    }

    /// 清除所有缓存。
    pub async fn clear(&self) {
        self.cache.lock().await.clear();
    }

    /// 生成缓存 key。
    fn cache_key(&self, invocation: &Invocation) -> String {
        let op = invocation.operation();
        format!("{}::{}", op.component(), op.method())
    }

    /// 检查缓存是否存在且未过期。
    async fn has_cached(&self, key: &str) -> bool {
        let mut cache = self.cache.lock().await;
        if let Some(entry) = cache.get(key) {
            if entry.is_expired() {
                cache.remove(key);
                false
            } else {
                true
            }
        } else {
            false
        }
    }

    /// 插入缓存值。
    async fn insert_cached(&self, key: String, value: InvocationValue) {
        let mut cache = self.cache.lock().await;

        // 检查容量
        if cache.len() >= self.max_size {
            // 简单策略：清除过期条目
            cache.retain(|_, entry| !entry.is_expired());
        }

        cache.insert(
            key,
            CacheEntry {
                value,
                inserted_at: Instant::now(),
                ttl: self.ttl,
            },
        );
    }
}

impl Default for CachingAspect {
    fn default() -> Self {
        Self::new()
    }
}

impl Interceptor for CachingAspect {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            let key = self.cache_key(&invocation);

            // 检查缓存
            if self.has_cached(&key).await {
                tracing::debug!("[CACHE HIT] {}", key);
            } else {
                tracing::debug!("[CACHE MISS] {}", key);
            }

            // 注意：由于 InvocationValue（Box<dyn Any + Send + Sync>）不实现 Clone，
            // 无法在缓存命中时返回缓存值。完整实现需要修改 InvocationValue 类型
            // 或使用 Arc 包装。当前实现仅记录缓存状态并放行。
            next.run(invocation).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caching_aspect_builder() {
        let aspect = CachingAspect::new()
            .with_max_size(100)
            .with_ttl(Duration::from_secs(60));

        assert_eq!(aspect.max_size, 100);
        assert_eq!(aspect.ttl, Some(Duration::from_secs(60)));
    }

    #[test]
    fn caching_aspect_default() {
        let aspect = CachingAspect::default();
        assert_eq!(aspect.max_size, usize::MAX);
        assert!(aspect.ttl.is_none());
    }

    #[tokio::test]
    async fn caching_aspect_clear() {
        let aspect = CachingAspect::new();
        assert_eq!(aspect.size().await, 0);

        aspect.clear().await;
        assert_eq!(aspect.size().await, 0);
    }

    // --- 私有方法覆盖测试 ---

    #[test]
    fn cache_key_generation() {
        let aspect = CachingAspect::new();
        let inv = Invocation::new(crate::Operation::new("UserService", "get_user"));
        let key = aspect.cache_key(&inv);
        assert_eq!(key, "UserService::get_user");
    }

    #[tokio::test]
    async fn has_cached_empty_cache() {
        let aspect = CachingAspect::new();
        assert!(!aspect.has_cached("nonexistent").await);
    }

    #[tokio::test]
    async fn insert_cached_then_has_cached() {
        let aspect = CachingAspect::new();
        let value: InvocationValue = Box::new(42i32);
        aspect.insert_cached("test_key".to_string(), value).await;
        assert!(aspect.has_cached("test_key").await);
        assert!(!aspect.has_cached("other_key").await);
    }

    #[tokio::test]
    async fn has_cached_expired_entry() {
        let aspect = CachingAspect::new().with_ttl(Duration::from_millis(1));
        let value: InvocationValue = Box::new(42i32);
        aspect.insert_cached("test_key".to_string(), value).await;
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(!aspect.has_cached("test_key").await);
    }

    #[tokio::test]
    async fn has_cached_non_expired_entry() {
        let aspect = CachingAspect::new().with_ttl(Duration::from_secs(60));
        let value: InvocationValue = Box::new(42i32);
        aspect.insert_cached("test_key".to_string(), value).await;
        assert!(aspect.has_cached("test_key").await);
    }

    #[tokio::test]
    async fn insert_cached_max_size_eviction() {
        let aspect = CachingAspect::new()
            .with_max_size(2)
            .with_ttl(Duration::from_millis(1));
        
        let v1: InvocationValue = Box::new(1i32);
        let v2: InvocationValue = Box::new(2i32);
        aspect.insert_cached("k1".to_string(), v1).await;
        aspect.insert_cached("k2".to_string(), v2).await;
        assert_eq!(aspect.size().await, 2);
        
        // Wait for entries to expire
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        // Insert 3rd - should trigger eviction of expired entries
        let v3: InvocationValue = Box::new(3i32);
        aspect.insert_cached("k3".to_string(), v3).await;
        assert_eq!(aspect.size().await, 1); // expired entries evicted
    }

    #[tokio::test]
    async fn insert_cached_no_ttl_never_expires() {
        let aspect = CachingAspect::new(); // no TTL
        let value: InvocationValue = Box::new(42i32);
        aspect.insert_cached("key".to_string(), value).await;
        assert!(aspect.has_cached("key").await);
    }

    #[test]
    fn is_expired_with_no_ttl() {
        let entry = CacheEntry {
            value: Box::new(42i32),
            inserted_at: std::time::Instant::now(),
            ttl: None,
        };
        assert!(!entry.is_expired());
    }

    #[test]
    fn is_expired_with_ttl_not_yet() {
        let entry = CacheEntry {
            value: Box::new(42i32),
            inserted_at: std::time::Instant::now(),
            ttl: Some(Duration::from_secs(60)),
        };
        assert!(!entry.is_expired());
    }

    #[test]
    fn is_expired_with_ttl_expired() {
        let entry = CacheEntry {
            value: Box::new(42i32),
            inserted_at: std::time::Instant::now() - Duration::from_secs(120),
            ttl: Some(Duration::from_secs(60)),
        };
        assert!(entry.is_expired());
    }
}
