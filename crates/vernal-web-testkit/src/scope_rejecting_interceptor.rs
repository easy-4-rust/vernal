//! Web 请求作用域策略拒绝拦截器对象。

use std::sync::Arc;

use vernal_aop::{
    Interceptor, Invocation, InvocationError, InvocationFuture, LocalInterceptor,
    LocalInvocationError, LocalInvocationFuture, LocalNext, Next,
};
use vernal_web::{ProblemDetails, ProblemKind, WebFailure, WebRequestScope};

use crate::ScopeCloseProbe;

/// 观察当前 AOP 调用的请求作用域，并返回框架中立的未认证失败。
///
/// 该对象同时实现 Send AOP 与 Local-AOP 契约，使 Tokio/Tower 风格 Adapter 和
/// Actix、Ntex 这类 Local Service Adapter 可以复用同一条错误生命周期断言。
/// Interceptor 不调用 `next`，因此还能验证策略短路时业务 Handler 不会被执行。
pub struct ScopeRejectingInterceptor {
    probe: Arc<ScopeCloseProbe>,
}

impl ScopeRejectingInterceptor {
    /// 创建绑定到单请求 Scope 观察器的拒绝拦截器。
    #[must_use]
    pub const fn new(probe: Arc<ScopeCloseProbe>) -> Self {
        Self { probe }
    }

    /// 从 AOP 强类型上下文取得 Adapter 写入的 Scope，并交给只读 Probe 观察。
    async fn observe_scope(&self, invocation: &Invocation) {
        let scope = invocation
            .context()
            .get::<Arc<WebRequestScope>>()
            .await
            .expect("Web AOP invocation must contain its request scope");
        self.probe.observe(&scope);
    }

    /// 构造所有 Adapter 都能映射成原生未认证响应的框架中立失败。
    fn unauthenticated_failure() -> WebFailure {
        WebFailure::new(ProblemDetails::new(
            ProblemKind::Unauthenticated,
            401,
            "Authentication is required",
        ))
    }
}

impl Interceptor for ScopeRejectingInterceptor {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        _next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            self.observe_scope(&invocation).await;
            Err(InvocationError::target(Self::unauthenticated_failure()))
        })
    }
}

impl LocalInterceptor for ScopeRejectingInterceptor {
    fn intercept_local<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        _next: LocalNext<'a>,
    ) -> LocalInvocationFuture<'a> {
        Box::pin(async move {
            self.observe_scope(&invocation).await;
            Err(LocalInvocationError::target(Self::unauthenticated_failure()))
        })
    }
}
