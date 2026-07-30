//! 拦截器模块。
//!
//! 包含核心 `Interceptor` trait 定义和 8 个业务切面。
//!
//! # 核心 Trait
//!
//! [`Interceptor`] 是 Vernal AOP 的核心环绕拦截器，对应 spring-aop `MethodInterceptor.invoke()`。
//!
//! # 业务切面
//!
//! 移植自 aspect-rs `aspect-std`，改造为 Tokio-first 异步。
//! 所有切面实现 `Interceptor` trait（而非 `Aspect`），因为需要 around 全控制。
//!
//! 对应 spring-aop：interceptor 包（13 个调试类）的 Rust 原生替代。

// 核心 Interceptor trait
mod interceptor_trait;

// 业务切面
pub mod logging_aspect;
pub mod timing_aspect;
pub mod metrics_aspect;
pub mod caching_aspect;
pub mod ratelimit_aspect;
pub mod circuitbreaker_aspect;
pub mod authorization_aspect;
pub mod validation_aspect;

// Re-export
pub use interceptor_trait::Interceptor;
pub use logging_aspect::LoggingAspect;
pub use timing_aspect::TimingAspect;
pub use metrics_aspect::MetricsAspect;
pub use caching_aspect::CachingAspect;
pub use ratelimit_aspect::RateLimitAspect;
pub use circuitbreaker_aspect::{CircuitBreakerAspect, CircuitState};
pub use authorization_aspect::{AuthorizationAspect, AllowlistAspect, AuthMode};
pub use validation_aspect::{ValidationAspect, ValidationRule};
