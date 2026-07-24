//! Salvo 请求生命周期与严格 AOP Hoop 对象。

use std::{pin::Pin, sync::Arc};

use salvo::{Depot, FlowCtrl, Handler, Request, Response, http::ResBody};
use tokio_util::sync::CancellationToken;
use vernal_aop::{BorrowedInvocationTarget, InvocationError};
use vernal_context::ApplicationContext;
use vernal_web::{HandlerInvocation, RequestContext, WebRequestScope};

use crate::{
    SalvoAopError, SalvoResponse, SalvoScopedBody, salvo_borrowed_target::SalvoBorrowedTarget,
    salvo_request_snapshot::SalvoRequestSnapshot, salvo_route_metadata::SalvoRouteMetadata,
};

/// 向 Salvo `Depot` 注入 Context，并让请求 Scope 跟随响应 Body。
#[derive(Clone)]
pub struct VernalSalvoHoop {
    context: Arc<ApplicationContext>,
    strict_aop: bool,
}

impl VernalSalvoHoop {
    /// 创建 Salvo Hoop。
    #[must_use]
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self {
            context,
            strict_aop: false,
        }
    }

    /// 创建同时管理请求 Scope 并执行严格 Send-AOP 的 Hoop。
    ///
    /// Salvo 必须启用 `matched-path`，Vernal 会在 Router 已完成匹配后使用低基数
    /// 路径模板和真实 HTTP 方法查找预编译计划。缺少模板或计划时 fail-closed，
    /// 不会回退到含用户输入的原始 URI。
    #[must_use]
    pub fn strict_aop(context: Arc<ApplicationContext>) -> Self {
        Self {
            context,
            strict_aop: true,
        }
    }

    /// 通过借用型 Send-AOP 目标驱动完整的 Salvo 下游 Handler 链。
    async fn call_with_aop(
        &self,
        request: &mut Request,
        depot: &mut Depot,
        response: &mut Response,
        control: &mut FlowCtrl,
        scope: Arc<WebRequestScope>,
        cancellation: CancellationToken,
    ) -> Result<(), SalvoAopError> {
        let route = SalvoRouteMetadata::capture(request)?;
        let operation = route.aop_operation();
        let snapshot = SalvoRequestSnapshot::capture(request);
        let request_context = Arc::new(RequestContext::new(route, cancellation));
        request_context.extensions().insert(snapshot).await;
        depot.inject(Arc::clone(&request_context));
        let invocation = HandlerInvocation::new(request_context, scope)
            .aop_invocation()
            .await;
        let plan = self
            .context
            .invocation_plans()
            .get(&operation)
            .cloned()
            .ok_or_else(|| {
                SalvoAopError::invocation(InvocationError::PlanNotFound {
                    operation: operation.clone(),
                })
            })?;

        // Salvo 的 Handler Future 满足 Send，但它独占借用四个框架对象。
        // BorrowedInvocationTarget 把这些借用约束在本次 plan.await 内，无需复制
        // Request/Response，也无需把 Salvo 错误语义改造成 Vernal 通用响应。
        let envelope = Arc::new(SalvoResponse::empty());
        let result = {
            let mut target =
                SalvoBorrowedTarget::new(request, depot, response, control, Arc::clone(&envelope));
            plan.invoke_borrowed(invocation, &mut target as &mut dyn BorrowedInvocationTarget)
                .await
        };

        let value = result.map_err(SalvoAopError::invocation)?;
        let returned = value
            .downcast::<Arc<SalvoResponse>>()
            .map_err(|_| SalvoAopError::ResponseTypeMismatch)?;
        let returned = *returned;
        *response = returned
            .take()
            .await
            .ok_or(SalvoAopError::ResponseUnavailable)?;
        Ok(())
    }
}

#[salvo::async_trait]
impl Handler for VernalSalvoHoop {
    async fn handle(
        &self,
        request: &mut Request,
        depot: &mut Depot,
        response: &mut Response,
        control: &mut FlowCtrl,
    ) {
        depot.inject(Arc::clone(&self.context));
        let cancellation = CancellationToken::new();
        let scope = Arc::new(WebRequestScope::new(cancellation.clone()));
        depot.inject(Arc::clone(&scope));

        // Handler Future 或响应 Body 被丢弃时，DropGuard 只负责同步发出取消信号；
        // 预先启动的 Tokio 任务负责执行真正的异步 Scope 关闭。
        let cleanup_scope = Arc::clone(&scope);
        let cleanup_cancellation = cancellation.clone();
        tokio::spawn(async move {
            cleanup_cancellation.cancelled().await;
            let _ = cleanup_scope.close().await;
        });

        let request_guard = cancellation.clone().drop_guard();
        if self.strict_aop {
            if let Err(error) = self
                .call_with_aop(
                    request,
                    depot,
                    response,
                    control,
                    Arc::clone(&scope),
                    cancellation,
                )
                .await
            {
                error.render(response);
            }
            // FlowCtrl 在当前 Hoop 返回后可能自动推进剩余 Handler。严格模式已经
            // 通过 AOP 目标完整推进或明确短路，因此必须终止外层自动推进。
            control.skip_rest();
        } else {
            control.call_next(request, depot, response).await;
        }

        // Salvo Handler 不用 Result 表达下游错误；无论状态码为何，都让 Scope
        // 跟随原生 ResBody 的 Frame 生命周期，而不是在 Handler 返回时提前关闭。
        let cancellation = request_guard.disarm();
        let body = response.take_body();
        response.body(ResBody::Boxed(Pin::from(Box::new(SalvoScopedBody::new(
            body,
            scope,
            cancellation,
        )))));
    }
}
