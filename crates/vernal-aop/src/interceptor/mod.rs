//! 拦截器模块。
//!
//! 包含核心 `Interceptor` trait 定义。
//!
//! # 核心 Trait
//!
//! [`Interceptor`] 是 Vernal AOP 的核心环绕拦截器，对应 spring-aop `MethodInterceptor.invoke()`。
//!
//! # 业务切面
//!
//! 业务切面直接使用 aspect-rs `aspect-std`，通过 `AspectRsAdapter` 适配为异步 Interceptor。
//!
//! 对应 spring-aop：interceptor 包（13 个调试类）的 Rust 原生替代。

// 核心 Interceptor trait
mod interceptor_trait;

// Re-export
pub use interceptor_trait::Interceptor;
