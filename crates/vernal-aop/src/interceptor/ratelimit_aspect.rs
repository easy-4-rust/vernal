//! 限流切面。
//!
//! 对应 aspect-rs：aspect-std/src/ratelimit.rs。
//! 语义参照 spring-aop：`ConcurrencyThrottleInterceptor`。
//!
//! 使用令牌桶算法限制函数调用频率。
//! 实现 `Interceptor` trait（around 全控制）。

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::{Interceptor, Invocation, InvocationError, InvocationFuture, InvocationResult, Next};

/// 限流切面。
///
/// 使用令牌桶算法限制函数调用频率。
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::RateLimitAspect;
/// use std::time::Duration;
///
/// // 每秒最多 10 次调用
/// let limiter = RateLimitAspect::new(10, Duration::from_secs(1));
/// ```
#[derive(Clone)]
pub struct RateLimitAspect {
    state: Arc<Mutex<RateLimitState>>,
}

struct RateLimitState {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64, // 每秒令牌数
    last_refill: Instant,
    per_function: bool,
    function_states: HashMap<String, FunctionRateLimit>,
}

struct FunctionRateLimit {
    tokens: f64,
    last_refill: Instant,
}

impl RateLimitAspect {
    /// 创建新的限流器。
    ///
    /// # 参数
    /// * `max_requests` - 允许的最大请求数
    /// * `window` - 时间窗口
    #[must_use]
    pub fn new(max_requests: u64, window: Duration) -> Self {
        let refill_rate = max_requests as f64 / window.as_secs_f64();

        Self {
            state: Arc::new(Mutex::new(RateLimitState {
                tokens: max_requests as f64,
                max_tokens: max_requests as f64,
                refill_rate,
                last_refill: Instant::now(),
                per_function: false,
                function_states: HashMap::new(),
            })),
        }
    }

    /// 启用按函数限流。每个函数独立拥有令牌桶。
    pub async fn with_per_function(self) -> Self {
        self.state.lock().await.per_function = true;
        self
    }

    /// 尝试获取令牌。
    async fn try_acquire(&self, function_name: Option<&str>) -> bool {
        let mut state = self.state.lock().await;
        let now = Instant::now();

        if state.per_function {
            if let Some(name) = function_name {
                let max_tokens = state.max_tokens;
                let refill_rate = state.refill_rate;

                let func_state = state
                    .function_states
                    .entry(name.to_string())
                    .or_insert_with(|| FunctionRateLimit {
                        tokens: max_tokens,
                        last_refill: now,
                    });

                // 补充令牌
                let elapsed = now.duration_since(func_state.last_refill).as_secs_f64();
                func_state.tokens = (func_state.tokens + elapsed * refill_rate).min(max_tokens);
                func_state.last_refill = now;

                if func_state.tokens >= 1.0 {
                    func_state.tokens -= 1.0;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            // 全局限流
            let elapsed = now.duration_since(state.last_refill).as_secs_f64();
            state.tokens = (state.tokens + elapsed * state.refill_rate).min(state.max_tokens);
            state.last_refill = now;

            if state.tokens >= 1.0 {
                state.tokens -= 1.0;
                true
            } else {
                false
            }
        }
    }

    /// 获取当前可用令牌数。
    pub async fn available_tokens(&self) -> f64 {
        let mut state = self.state.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_refill).as_secs_f64();
        state.tokens = (state.tokens + elapsed * state.refill_rate).min(state.max_tokens);
        state.last_refill = now;
        state.tokens
    }
}

impl Interceptor for RateLimitAspect {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            let op = invocation.operation();
            let function_key = format!("{}::{}", op.component(), op.method());
            let allowed = self.try_acquire(Some(&function_key)).await;

            if allowed {
                next.run(invocation).await
            } else {
                Err(InvocationError::Cancelled)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rate_limit_basic() {
        let limiter = RateLimitAspect::new(5, Duration::from_secs(1));

        // 应允许 5 次调用
        for _ in 0..5 {
            assert!(limiter.try_acquire(Some("test")).await);
        }

        // 第 6 次应被拒绝
        assert!(!limiter.try_acquire(Some("test")).await);
    }

    #[tokio::test]
    async fn rate_limit_refill() {
        let limiter = RateLimitAspect::new(2, Duration::from_millis(100));

        assert!(limiter.try_acquire(Some("test")).await);
        assert!(limiter.try_acquire(Some("test")).await);
        assert!(!limiter.try_acquire(Some("test")).await);

        // 等待补充
        tokio::time::sleep(Duration::from_millis(150)).await;

        assert!(limiter.try_acquire(Some("test")).await);
    }

    #[tokio::test]
    async fn per_function_limiting() {
        let limiter = RateLimitAspect::new(2, Duration::from_secs(1))
            .with_per_function()
            .await;

        // 函数 A 消耗配额
        assert!(limiter.try_acquire(Some("func_a")).await);
        assert!(limiter.try_acquire(Some("func_a")).await);
        assert!(!limiter.try_acquire(Some("func_a")).await);

        // 函数 B 应有独立配额
        assert!(limiter.try_acquire(Some("func_b")).await);
        assert!(limiter.try_acquire(Some("func_b")).await);
        assert!(!limiter.try_acquire(Some("func_b")).await);
    }

    #[tokio::test]
    async fn available_tokens() {
        let limiter = RateLimitAspect::new(10, Duration::from_secs(1));

        let initial = limiter.available_tokens().await;
        assert!((initial - 10.0).abs() < 0.01);

        limiter.try_acquire(Some("test")).await;

        let after = limiter.available_tokens().await;
        assert!((after - 9.0).abs() < 0.01);
    }
}
