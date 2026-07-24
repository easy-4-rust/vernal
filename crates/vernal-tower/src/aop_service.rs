//! Tower AOP Service 对象。

use std::{
    any::type_name,
    error::Error,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use http::Request;
use tokio::sync::Mutex;
use tower::Service;
use vernal_aop::{InvocationError, InvocationTarget, InvocationValue};
use vernal_context::ApplicationContext;
use vernal_http::HttpRequestSnapshot;
use vernal_web::{HandlerInvocation, RequestContext, WebRequestScope};

use crate::tower_upstream_error::TowerUpstreamError;
use crate::{AopServiceError, MissingPlanPolicy, TowerResponse, TowerRouteResolver};

/// 执行 Vernal AOP 调用计划的 Tower `Service`。
///
/// 该对象不读取或重建请求/响应 Body。请求所有权只在最终目标真正执行时移交给
/// 下游 Service；拦截器短路时下游不会被调用。响应通过 [`TowerResponse`] 在
/// 类型擦除边界内传递，从而保留具体框架的原生响应类型。
#[derive(Clone)]
pub struct AopService<S, R> {
    inner: S,
    resolver: R,
    missing_plan_policy: MissingPlanPolicy,
}

impl<S, R> AopService<S, R> {
    /// 由 [`crate::AopLayer`] 创建 AOP Service。
    pub(crate) const fn new(inner: S, resolver: R, missing_plan_policy: MissingPlanPolicy) -> Self {
        Self {
            inner,
            resolver,
            missing_plan_policy,
        }
    }
}

impl<S, R, B> Service<Request<B>> for AopService<S, R>
where
    S: Service<Request<B>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Response: Send + 'static,
    S::Error: Error + Send + Sync + 'static,
    R: TowerRouteResolver<B>,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = AopServiceError<S::Error>;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, context: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner
            .poll_ready(context)
            .map_err(AopServiceError::Upstream)
    }

    fn call(&mut self, mut request: Request<B>) -> Self::Future {
        // AOP Layer 必须处在 Context 与 RequestScope Layer 之后；缺失依赖时
        // 立即失败，避免请求在半初始化状态下进入业务 Handler。
        let Some(application_context) = request
            .extensions()
            .get::<Arc<ApplicationContext>>()
            .cloned()
        else {
            return Box::pin(async { Err(AopServiceError::MissingApplicationContext) });
        };
        let Some(request_scope) = request.extensions().get::<Arc<WebRequestScope>>().cloned()
        else {
            return Box::pin(async { Err(AopServiceError::MissingRequestScope) });
        };

        // ContextPropagationLayer 或具体框架适配器可以预先提供完整请求上下文；
        // 否则由当前 Layer 使用稳定路由元数据和 Scope 取消令牌创建。
        let request_context =
            if let Some(context) = request.extensions().get::<Arc<RequestContext>>().cloned() {
                context
            } else {
                let Some(route) = self.resolver.resolve(&request) else {
                    return Box::pin(async { Err(AopServiceError::MissingRouteMetadata) });
                };
                let context = Arc::new(RequestContext::new(
                    route,
                    request_scope.cancellation().clone(),
                ));
                request.extensions_mut().insert(Arc::clone(&context));
                context
            };

        let operation = request_context.route().aop_operation();
        let request_snapshot = HttpRequestSnapshot::capture(&request);
        let plan = application_context
            .invocation_plans()
            .get(&operation)
            .cloned();

        // `Proceed` 必须由应用显式选择；默认严格模式在此处 fail closed。
        if plan.is_none() && self.missing_plan_policy == MissingPlanPolicy::Reject {
            return Box::pin(async move {
                Err(AopServiceError::Invocation(InvocationError::PlanNotFound {
                    operation,
                }))
            });
        }

        // Tower 要求 call 使用刚刚通过 poll_ready 的 Service。用 clone 替换字段，
        // 把 ready 实例连同 owned Request 一起移入异步目标。
        let clone = self.inner.clone();
        let inner = std::mem::replace(&mut self.inner, clone);
        let Some(plan) = plan else {
            return Box::pin(async move {
                ensure_request_snapshot(&request_context, request_snapshot).await;
                let mut inner = inner;
                inner.call(request).await.map_err(AopServiceError::Upstream)
            });
        };

        let handler_invocation =
            HandlerInvocation::new(Arc::clone(&request_context), Arc::clone(&request_scope));
        let call_state = Arc::new(Mutex::new(Some((inner, request))));
        let target_state = Arc::clone(&call_state);
        let target_operation = operation.clone();
        let target: Arc<InvocationTarget> = Arc::new(move |_invocation| {
            let target_state = Arc::clone(&target_state);
            let target_operation = target_operation.clone();
            Box::pin(async move {
                let Some((mut inner, request)) = target_state.lock().await.take() else {
                    return Err(InvocationError::TargetAlreadyInvoked {
                        operation: target_operation,
                    });
                };

                inner
                    .call(request)
                    .await
                    .map(|response| Box::new(TowerResponse::new(response)) as InvocationValue)
                    .map_err(|source| InvocationError::target(TowerUpstreamError(source)))
            })
        });

        Box::pin(async move {
            ensure_request_snapshot(&request_context, request_snapshot).await;
            let invocation = handler_invocation.aop_invocation().await;
            let value = match plan.invoke(invocation, target).await {
                Ok(value) => value,
                Err(error) => {
                    return match error.into_target::<TowerUpstreamError<S::Error>>() {
                        Ok(TowerUpstreamError(source)) => Err(AopServiceError::Upstream(source)),
                        Err(error) => Err(AopServiceError::Invocation(error)),
                    };
                }
            };

            let response = value
                .downcast::<TowerResponse<S::Response>>()
                .map_err(|_| {
                    AopServiceError::Invocation(InvocationError::ReturnTypeMismatch {
                        expected: type_name::<TowerResponse<S::Response>>(),
                    })
                })?;
            response
                .take()
                .await
                .ok_or(AopServiceError::ResponseAlreadyTaken)
        })
    }
}

/// 确保安全与审计拦截器能读取 owned HTTP 元数据，且不覆盖上层 Adapter 已写入
/// 的更精确快照。
async fn ensure_request_snapshot(
    request_context: &RequestContext,
    request_snapshot: HttpRequestSnapshot,
) {
    if !request_context
        .extensions()
        .contains::<HttpRequestSnapshot>()
        .await
    {
        request_context.extensions().insert(request_snapshot).await;
    }
}
