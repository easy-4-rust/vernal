//! 测试用 gRPC 安全策略拒绝拦截器对象。

use std::sync::Arc;

use vernal_aop::{Interceptor, Invocation, InvocationError, InvocationFuture, Next};
use vernal_web::{ProblemDetails, ProblemKind, WebFailure};

/// 返回未认证 Web 失败，验证 Tonic 映射为 gRPC Status 而非传输错误。
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
