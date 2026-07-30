//! 熔断切面。
//!
//! 对应 aspect-rs：aspect-std/src/circuitbreaker.rs。
//! spring-aop 无直接对应。
//!
//! 实现熔断器模式，防止级联故障。
//! 实现 `Interceptor` trait（around 全控制）。

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::{Interceptor, Invocation, InvocationError, InvocationFuture, InvocationResult, Next};

/// 熔断器状态。
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// 关闭状态：正常放行，记录失败次数。
    Closed,
    /// 打开状态：快速失败，不调用目标函数。
    Open { until: Instant },
    /// 半开状态：试探性放行，测试服务是否恢复。
    HalfOpen,
}

/// 熔断切面。
///
/// 实现熔断器模式，防止级联故障。
///
/// # 状态转换
/// - **Closed**: 正常运行，跟踪失败次数
/// - **Open**: 超过阈值后进入快速失败模式
/// - **Half-Open**: 超时后试探性放行
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::CircuitBreakerAspect;
/// use std::time::Duration;
///
/// // 5 次失败后熔断，30 秒后尝试恢复
/// let breaker = CircuitBreakerAspect::new(5, Duration::from_secs(30));
/// ```
#[derive(Clone)]
pub struct CircuitBreakerAspect {
    state: Arc<Mutex<CircuitBreakerState>>,
}

struct CircuitBreakerState {
    circuit_state: CircuitState,
    failure_count: usize,
    success_count: usize,
    failure_threshold: usize,
    timeout: Duration,
    half_open_max_requests: usize,
}

impl CircuitBreakerAspect {
    /// 创建新的熔断器。
    ///
    /// # 参数
    /// * `failure_threshold` - 触发熔断的失败次数
    /// * `timeout` - 熔断持续时间
    #[must_use]
    pub fn new(failure_threshold: usize, timeout: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitBreakerState {
                circuit_state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                failure_threshold,
                timeout,
                half_open_max_requests: 1,
            })),
        }
    }

    /// 设置半开状态下的最大请求数。
    pub async fn with_half_open_requests(self, max_requests: usize) -> Self {
        self.state.lock().await.half_open_max_requests = max_requests;
        self
    }

    /// 获取当前熔断状态。
    pub async fn state(&self) -> CircuitState {
        self.state.lock().await.circuit_state.clone()
    }

    /// 手动重置熔断器到关闭状态。
    pub async fn reset(&self) {
        let mut state = self.state.lock().await;
        state.circuit_state = CircuitState::Closed;
        state.failure_count = 0;
        state.success_count = 0;
    }

    /// 记录成功调用。
    async fn record_success(&self) {
        let mut state = self.state.lock().await;

        match state.circuit_state {
            CircuitState::HalfOpen => {
                state.success_count += 1;
                if state.success_count >= state.half_open_max_requests {
                    state.circuit_state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                }
            }
            CircuitState::Closed => {
                state.failure_count = 0;
            }
            CircuitState::Open { .. } => {
                state.failure_count = 0;
                state.success_count = 0;
            }
        }
    }

    /// 记录失败调用。
    async fn record_failure(&self) {
        let mut state = self.state.lock().await;

        match state.circuit_state {
            CircuitState::HalfOpen => {
                state.circuit_state = CircuitState::Open {
                    until: Instant::now() + state.timeout,
                };
                state.success_count = 0;
            }
            CircuitState::Closed => {
                state.failure_count += 1;
                if state.failure_count >= state.failure_threshold {
                    state.circuit_state = CircuitState::Open {
                        until: Instant::now() + state.timeout,
                    };
                }
            }
            CircuitState::Open { .. } => {}
        }
    }

    /// 检查是否允许请求通过。
    async fn should_allow_request(&self) -> Result<(), InvocationError> {
        let mut state = self.state.lock().await;

        match state.circuit_state {
            CircuitState::Closed => Ok(()),
            CircuitState::HalfOpen => Ok(()),
            CircuitState::Open { until } => {
                if Instant::now() >= until {
                    state.circuit_state = CircuitState::HalfOpen;
                    state.success_count = 0;
                    Ok(())
                } else {
                    Err(InvocationError::Cancelled)
                }
            }
        }
    }
}

impl Interceptor for CircuitBreakerAspect {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            self.should_allow_request().await?;

            match next.run(invocation).await {
                Ok(result) => {
                    self.record_success().await;
                    Ok(result)
                }
                Err(e) => {
                    self.record_failure().await;
                    Err(e)
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn circuit_breaker_closed_initially() {
        let breaker = CircuitBreakerAspect::new(3, Duration::from_secs(1));
        assert_eq!(breaker.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn circuit_opens_after_threshold() {
        let breaker = CircuitBreakerAspect::new(3, Duration::from_secs(60));

        for _ in 0..3 {
            breaker.record_failure().await;
        }

        match breaker.state().await {
            CircuitState::Open { .. } => (),
            _ => panic!("Circuit should be open"),
        }
    }

    #[tokio::test]
    async fn circuit_rejects_when_open() {
        let breaker = CircuitBreakerAspect::new(1, Duration::from_secs(60));

        breaker.record_failure().await;
        assert!(breaker.should_allow_request().await.is_err());
    }

    #[tokio::test]
    async fn circuit_transitions_to_half_open() {
        let breaker = CircuitBreakerAspect::new(1, Duration::from_millis(100));

        breaker.record_failure().await;
        assert!(matches!(breaker.state().await, CircuitState::Open { .. }));

        tokio::time::sleep(Duration::from_millis(150)).await;

        assert!(breaker.should_allow_request().await.is_ok());
        assert_eq!(breaker.state().await, CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn circuit_closes_after_success() {
        let breaker = CircuitBreakerAspect::new(1, Duration::from_millis(50));

        breaker.record_failure().await;
        tokio::time::sleep(Duration::from_millis(60)).await;

        breaker.should_allow_request().await.unwrap();
        breaker.record_success().await;
        assert_eq!(breaker.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn circuit_reset() {
        let breaker = CircuitBreakerAspect::new(2, Duration::from_secs(60));

        breaker.record_failure().await;
        breaker.record_failure().await;
        assert!(matches!(breaker.state().await, CircuitState::Open { .. }));

        breaker.reset().await;
        assert_eq!(breaker.state().await, CircuitState::Closed);
    }
}
