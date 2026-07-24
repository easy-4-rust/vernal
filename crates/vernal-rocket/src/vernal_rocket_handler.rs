//! Rocket Route 严格 AOP Handler 包装对象。

use std::sync::Arc;

use rocket::{
    Data, Request,
    route::{Handler, Outcome},
};
use vernal_aop::{BorrowedInvocationTarget, InvocationError};
use vernal_context::ApplicationContext;
use vernal_web::{HandlerInvocation, RequestContext, RouteMetadata};

use crate::{
    RocketAopError, rocket_borrowed_target::RocketBorrowedTarget,
    rocket_outcome_marker::RocketOutcomeMarker, rocket_request_snapshot::RocketRequestSnapshot,
    rocket_request_state::RocketRequestState,
};

/// 在不修改业务函数签名的前提下包裹 Rocket 原生 Route Handler。
///
/// 包装器保留 Rocket 的 `Success`、`Error` 与 `Forward` 三态 Outcome。AOP 目标
/// 持有一次性 Data，并借用 Request 与原 Handler；这些带请求生命周期的对象不会
/// 逃逸到 `'static` 调用上下文。
#[derive(Clone)]
pub(crate) struct VernalRocketHandler {
    inner: Box<dyn Handler>,
}

impl VernalRocketHandler {
    /// 创建保留原生 Handler 行为的严格 AOP 包装器。
    pub(crate) const fn new(inner: Box<dyn Handler>) -> Self {
        Self { inner }
    }
}

#[rocket::async_trait]
impl Handler for VernalRocketHandler {
    async fn handle<'r>(&self, request: &'r Request<'_>, data: Data<'r>) -> Outcome<'r> {
        let Some(context) = request.rocket().state::<Arc<ApplicationContext>>().cloned() else {
            return RocketAopError::MissingContext.outcome(request);
        };
        let request_state = request.local_cache(RocketRequestState::missing);
        let Some(scope) = request_state.scope() else {
            return RocketAopError::MissingRequestScope.outcome(request);
        };
        let Some(matched_route) = request.route() else {
            return RocketAopError::MissingRouteMetadata.outcome(request);
        };
        let snapshot = match RocketRequestSnapshot::capture(request) {
            Ok(snapshot) => snapshot,
            Err(error) => return error.outcome(request),
        };

        // 必须从运行时已匹配 Route 读取模板：Rocket::mount 会给宏生成的原始 URI
        // 加上挂载基路径。若在 with_vernal_aop 阶段提前缓存，"/api" 等前缀会丢失，
        // 从而造成 InvocationPlan 误匹配。这里得到的是完整且仍为低基数的 URI 模板。
        let path_pattern: Arc<str> = matched_route.uri.path().into();
        let route = RouteMetadata::new(
            Arc::clone(&path_pattern),
            request.method().as_str().to_owned(),
            path_pattern,
        );
        let operation = route.aop_operation();
        let request_context = Arc::new(RequestContext::new(route, scope.cancellation().clone()));
        request_context.extensions().insert(snapshot).await;
        request_state.set_request_context(Arc::clone(&request_context));

        let Some(plan) = context.invocation_plans().get(&operation).cloned() else {
            return RocketAopError::invocation(InvocationError::PlanNotFound { operation })
                .outcome(request);
        };
        let invocation = HandlerInvocation::new(request_context, scope)
            .aop_invocation()
            .await;
        let mut target = RocketBorrowedTarget::new(self.inner.as_ref(), request, data);
        let result = plan
            .invoke_borrowed(invocation, &mut target as &mut dyn BorrowedInvocationTarget)
            .await;

        match result {
            Ok(value) if value.is::<Arc<RocketOutcomeMarker>>() => target
                .take_outcome()
                .unwrap_or_else(|| RocketAopError::OutcomeUnavailable.outcome(request)),
            Ok(_) => RocketAopError::OutcomeTypeMismatch.outcome(request),
            Err(error) => RocketAopError::invocation(error).outcome(request),
        }
    }
}
