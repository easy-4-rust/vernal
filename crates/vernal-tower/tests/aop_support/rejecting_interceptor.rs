//! 测试用拒绝拦截器对象。

use std::{io, sync::Arc};

use vernal_aop::{Interceptor, Invocation, InvocationError, InvocationFuture, Next};

/// 使用与下游相同的 `io::Error` 类型拒绝请求。
pub struct RejectingInterceptor;

impl Interceptor for RejectingInterceptor {
    fn intercept<'a>(
        &'a self,
        _invocation: Arc<Invocation>,
        _next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async {
            Err(InvocationError::target(io::Error::other(
                "interceptor rejected",
            )))
        })
    }
}
