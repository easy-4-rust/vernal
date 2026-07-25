//! 无状态透传拦截器对象。

use std::sync::Arc;

use vernal_aop::{Interceptor, Invocation, InvocationFuture, Next};

/// 只推进 `Next`、不执行额外业务逻辑的环绕拦截器。
///
/// 使用该对象可以单独观察动态分派、Box Future 和链节点推进的边际成本。
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PassthroughInterceptor;

impl Interceptor for PassthroughInterceptor {
    /// 将当前调用原样交给下一个拦截器或最终目标。
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        next.run(invocation)
    }
}
