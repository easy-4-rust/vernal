#![forbid(unsafe_code)]
#![doc = "Vernal 内建 AOP 切面。"]
//!
//! 对标 Spring 的 `spring-aspects` 模块，提供框架级横切关注点实现：
//!
//! - `TransactionalAspect`：事务管理（对标 `@Transactional`）
//! - `CacheableAspect`：缓存管理（对标 `@Cacheable`）
//! - `AsyncAspect`：异步执行（对标 `@Async`）
//! - `ScheduledAspect`：定时调度（对标 `@Scheduled`）
//!
//! ## 设计原则
//!
//! - 所有切面都是 `vernal-aop::Interceptor` 的实现
//! - 切面通过 `ApplicationModule` 注册到 ApplicationContext
//! - 切面不直接依赖具体实现（事务/缓存等由 `vernal-tx`/`vernal-cache` 提供）
//! - 切面只负责 AOP 拦截逻辑，不负责底层实现

mod transactional_aspect;
mod cacheable_aspect;
mod async_aspect;
mod scheduled_aspect;

pub use transactional_aspect::TransactionalAspect;
pub use cacheable_aspect::CacheableAspect;
pub use async_aspect::AsyncAspect;
pub use scheduled_aspect::ScheduledAspect;
