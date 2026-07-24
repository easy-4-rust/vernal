//! 启动报告测试使用的 Local-AOP 透传拦截器。

use std::sync::Arc;

use vernal_aop::{Invocation, LocalInterceptor, LocalInvocationFuture, LocalNext};

/// 不改变本地调用结果、只用于形成一个 Local-AOP 计划槽位的拦截器。
pub struct PassThroughLocalInterceptor;

impl LocalInterceptor for PassThroughLocalInterceptor {
    fn intercept_local<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: LocalNext<'a>,
    ) -> LocalInvocationFuture<'a> {
        next.run(invocation)
    }
}
