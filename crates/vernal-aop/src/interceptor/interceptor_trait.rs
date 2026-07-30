//! 异步环绕拦截器契约。
//!
//! 对应 spring-aop：`MethodInterceptor.invoke()`。

use std::sync::Arc;

use crate::{Invocation, InvocationFuture, Next};

/// Vernal AOP 的核心环绕拦截器。
///
/// 实现者可以在调用 [`Next::run`] 前后执行逻辑，也可以不调用 `next` 直接返回，
/// 形成鉴权拒绝、缓存命中或熔断短路。结果与错误均可在返回前被替换。
pub trait Interceptor: Send + Sync + 'static {
    /// 环绕当前调用。
    fn intercept<'a>(&'a self, invocation: Arc<Invocation>, next: Next<'a>)
    -> InvocationFuture<'a>;
}
