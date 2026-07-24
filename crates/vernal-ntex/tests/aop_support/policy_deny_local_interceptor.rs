//! 测试用 Ntex Local-AOP 安全策略拒绝拦截器对象。

use std::sync::Arc;

use vernal_aop::{
    Invocation, LocalInterceptor, LocalInvocationError, LocalInvocationFuture, LocalNext,
};
use vernal_web::{ProblemDetails, ProblemKind, WebFailure};

/// 返回框架中立的未认证失败，不推进下游 Ntex Service。
pub struct PolicyDenyLocalInterceptor;

impl LocalInterceptor for PolicyDenyLocalInterceptor {
    fn intercept_local<'a>(
        &'a self,
        _invocation: Arc<Invocation>,
        _next: LocalNext<'a>,
    ) -> LocalInvocationFuture<'a> {
        Box::pin(async {
            Err(LocalInvocationError::target(WebFailure::new(
                ProblemDetails::new(
                    ProblemKind::Unauthenticated,
                    401,
                    "Authentication is required",
                ),
            )))
        })
    }
}
