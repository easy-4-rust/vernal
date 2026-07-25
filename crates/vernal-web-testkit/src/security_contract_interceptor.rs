//! Web 安全桥一致性合同拦截器。

use std::sync::Arc;

use vernal_aop::{
    Interceptor, Invocation, InvocationError, InvocationFuture, LocalInterceptor,
    LocalInvocationError, LocalInvocationFuture, LocalNext, Next,
};
use vernal_web::{ProblemDetails, ProblemKind, RequestContext, SecurityPrincipal, WebFailure};

/// 模拟消费方安全桥对 Vernal 请求上下文产生的最小、框架中立结果。
///
/// 本对象不解析 Token、不读取 Session，也不判断 Sa-Token 权限通配符；这些语义
/// 必须继续由 Sa-Token-Rust 等安全实现拥有。它只冻结 Vernal Adapter 必须支持的
/// 三项边界合同：
///
/// 1. 认证成功后把只读 [`SecurityPrincipal`] 写入当前 [`RequestContext`]；
/// 2. 匿名访问受保护操作时以稳定、脱敏的 401 `WebFailure` 短路；
/// 3. 已认证但授权不足时保留 Principal，并以稳定的 403 `WebFailure` 短路。
///
/// 同一个对象同时实现 Send-AOP 与 Local-AOP，确保线程安全和 Worker-local Web
/// 框架使用完全一致的安全投影与拒绝语义。
#[derive(Clone)]
pub struct SecurityContractInterceptor {
    principal: Option<Arc<SecurityPrincipal>>,
    denial: Option<ProblemDetails>,
}

impl SecurityContractInterceptor {
    /// 创建认证成功且允许继续执行目标的合同拦截器。
    #[must_use]
    pub fn authenticated<I, S>(subject: impl Into<Arc<str>>, roles: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Arc<str>>,
    {
        Self {
            principal: Some(Arc::new(SecurityPrincipal::new(subject, roles))),
            denial: None,
        }
    }

    /// 创建匿名访问受保护操作的 401 合同拦截器。
    #[must_use]
    pub fn unauthenticated() -> Self {
        Self {
            principal: None,
            denial: Some(ProblemDetails::new(
                ProblemKind::PolicyDenied,
                401,
                "Authentication is required",
            )),
        }
    }

    /// 创建已认证但无权访问操作的 403 合同拦截器。
    #[must_use]
    pub fn forbidden<I, S>(subject: impl Into<Arc<str>>, roles: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Arc<str>>,
    {
        Self {
            principal: Some(Arc::new(SecurityPrincipal::new(subject, roles))),
            denial: Some(ProblemDetails::new(
                ProblemKind::PolicyDenied,
                403,
                "Access is forbidden",
            )),
        }
    }

    /// 投影 Principal，并在合同要求拒绝时返回框架中立失败。
    ///
    /// 缺少 `RequestContext` 表示 Adapter 没有建立安全桥所需的调用边界，因此按
    /// 基础设施错误 fail-closed；错误文本保持静态，不携带 Token、Header 或用户数据。
    async fn apply(&self, invocation: &Arc<Invocation>) -> Result<(), WebFailure> {
        let Some(request_context) = invocation.context().get::<Arc<RequestContext>>().await else {
            return Err(WebFailure::new(ProblemDetails::new(
                ProblemKind::Infrastructure,
                500,
                "Vernal request context is unavailable",
            )));
        };

        // Principal 写入发生在授权拒绝之前，使后续诊断与消费方错误处理仍能区分
        // “未认证”与“已认证但无权限”，同时不会把具体凭证写入协议错误。
        request_context
            .set_principal(self.principal.as_ref().map(Arc::clone))
            .await;
        match &self.denial {
            Some(problem) => Err(WebFailure::new(problem.clone())),
            None => Ok(()),
        }
    }
}

impl Interceptor for SecurityContractInterceptor {
    /// 在线程安全调用链中投影安全结果并按合同推进或短路目标。
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            self.apply(&invocation)
                .await
                .map_err(InvocationError::target)?;
            next.run(invocation).await
        })
    }
}

impl LocalInterceptor for SecurityContractInterceptor {
    /// 在 Worker-local 调用链中复用完全相同的安全投影与拒绝逻辑。
    fn intercept_local<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: LocalNext<'a>,
    ) -> LocalInvocationFuture<'a> {
        Box::pin(async move {
            self.apply(&invocation)
                .await
                .map_err(LocalInvocationError::target)?;
            next.run(invocation).await
        })
    }
}
