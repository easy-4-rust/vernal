//! 测试用本地短路拦截器对象。

use std::{rc::Rc, sync::Arc};

use vernal_aop::{
    Invocation, LocalInterceptor, LocalInvocationFuture, LocalInvocationValue, LocalNext,
};

/// 不推进目标，直接返回一个 `Rc` 值。
pub struct ShortCircuitLocalInterceptor;

impl LocalInterceptor for ShortCircuitLocalInterceptor {
    fn intercept_local<'a>(
        &'a self,
        _invocation: Arc<Invocation>,
        _next: LocalNext<'a>,
    ) -> LocalInvocationFuture<'a> {
        Box::pin(async { Ok(Box::new(Rc::new(String::from("blocked"))) as LocalInvocationValue) })
    }
}
