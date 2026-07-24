//! 本地异步环绕拦截器契约。

use std::sync::Arc;

use crate::{Invocation, LocalInvocationFuture, LocalNext};

/// 面向 `!Send` 目标 Future 的环绕拦截器。
///
/// 拦截器对象本身仍要求 `Send + Sync`，因此不可变计划可以进入
/// `ApplicationContext` 并被所有 Worker 共享；只有每次调用产生的 Future、
/// 目标值和错误保持线程本地。实现者可以短路、替换结果或环绕完整目标 Future。
pub trait LocalInterceptor: Send + Sync + 'static {
    /// 环绕当前本地调用。
    fn intercept_local<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: LocalNext<'a>,
    ) -> LocalInvocationFuture<'a>;
}
