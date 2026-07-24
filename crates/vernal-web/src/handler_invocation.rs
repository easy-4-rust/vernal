//! Web Handler 调用对象。

use std::sync::Arc;

use vernal_aop::Invocation;

use crate::{RequestContext, WebRequestScope};

/// 将路由上下文和请求作用域绑定到一次 Handler 调用。
pub struct HandlerInvocation {
    context: Arc<RequestContext>,
    scope: Arc<WebRequestScope>,
}

impl HandlerInvocation {
    /// 创建 Handler 调用。
    #[must_use]
    pub fn new(context: Arc<RequestContext>, scope: Arc<WebRequestScope>) -> Self {
        Self { context, scope }
    }

    /// 构建共享相同取消/deadline 的 AOP 调用，并写入 Context 与 Scope 扩展。
    pub async fn aop_invocation(&self) -> Arc<Invocation> {
        let mut invocation = Invocation::new(self.context.route().aop_operation())
            .with_cancellation(self.context.cancellation().clone());
        if let Some(deadline) = self.context.deadline() {
            invocation = invocation.with_deadline(deadline);
        }
        let invocation = invocation.shared();
        invocation.context().insert(Arc::clone(&self.context)).await;
        invocation.context().insert(Arc::clone(&self.scope)).await;
        invocation
    }

    /// 返回请求上下文。
    #[must_use]
    pub const fn context(&self) -> &Arc<RequestContext> {
        &self.context
    }

    /// 返回请求作用域。
    #[must_use]
    pub const fn scope(&self) -> &Arc<WebRequestScope> {
        &self.scope
    }
}
