//! 测试用 Salvo 安全策略拒绝拦截器对象。

use std::sync::Arc;

use vernal_aop::{Interceptor, Invocation, InvocationError, InvocationFuture, Next};
use vernal_web::{ProblemDetails, ProblemKind, WebFailure};

/// 返回框架中立的未认证失败，不推进下游 Salvo Handler。
pub struct PolicyDenyInterceptor;

impl Interceptor for PolicyDenyInterceptor {
    fn intercept<'a>(
        &'a self,
        _invocation: Arc<Invocation>,
        _next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async {
            Err(InvocationError::target(WebFailure::new(
                ProblemDetails::new(
                    ProblemKind::Unauthenticated,
                    401,
                    "Authentication is required",
                ),
            )))
        })
    }
}
