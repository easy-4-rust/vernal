//! 启动报告测试使用的透传拦截器。

use std::sync::Arc;

use vernal_aop::{Interceptor, Invocation, InvocationFuture, Next};

/// 不改变调用结果、只用于形成一个真实 AOP 计划槽位的拦截器。
pub struct PassThroughInterceptor;

impl Interceptor for PassThroughInterceptor {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        next.run(invocation)
    }
}
