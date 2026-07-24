//! Gotham 请求生命周期与严格 AOP Middleware 对象。

use std::{panic::AssertUnwindSafe, pin::Pin, sync::Arc};

use gotham::{
    handler::{HandlerError, HandlerFuture, HandlerResult},
    helpers::http::Body,
    middleware::{Middleware, NewMiddleware},
    state::{FromState, State},
};
use http::{HeaderMap, Method, Response, Uri, Version};
use http_body_util::BodyExt;
use tokio_util::sync::CancellationToken;
use vernal_aop::{BorrowedInvocationTarget, InvocationError};
use vernal_context::ApplicationContext;
use vernal_http::HttpRequestSnapshot;
use vernal_web::{HandlerInvocation, RequestContext, RouteMetadata, WebRequestScope};

use crate::{
    GothamAopError, GothamRejection, GothamResponse, GothamScopedBody, VernalGothamContext,
    VernalGothamRequestContext, VernalGothamRequestScope,
    gotham_borrowed_target::GothamBorrowedTarget, gotham_upstream_error::GothamUpstreamError,
};

/// 为 Gotham Pipeline 注入 Context，并让请求 Scope 跟随响应 Body。
pub struct VernalGothamMiddleware {
    // Gotham 只在中间件工厂引用跨越 panic 捕获边界时要求 RefUnwindSafe。
    // Context 内部状态仍由 Tokio 锁保护；这里不绕过任何线程安全约束。
    context: AssertUnwindSafe<Arc<ApplicationContext>>,
    strict_aop_path_pattern: Option<Arc<str>>,
}

impl Clone for VernalGothamMiddleware {
    fn clone(&self) -> Self {
        Self {
            context: AssertUnwindSafe(Arc::clone(&self.context.0)),
            strict_aop_path_pattern: self.strict_aop_path_pattern.clone(),
        }
    }
}

impl VernalGothamMiddleware {
    /// 创建 Gotham 中间件。
    #[must_use]
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self {
            context: AssertUnwindSafe(context),
            strict_aop_path_pattern: None,
        }
    }

    /// 创建同时管理请求 Scope 并执行严格 Send-AOP 的 Middleware。
    ///
    /// Gotham 的公开 State 不包含最终匹配的路由模板。调用方应把中间件放入服务
    /// 该具体路由的 Pipeline，并传入相同的完整低基数模式。空模式或缺少预编译
    /// 计划时 fail-closed，不回退到包含用户输入的 URI。
    #[must_use]
    pub fn strict_aop(context: Arc<ApplicationContext>, path_pattern: impl Into<Arc<str>>) -> Self {
        Self {
            context: AssertUnwindSafe(context),
            strict_aop_path_pattern: Some(path_pattern.into()),
        }
    }

    /// 通过借用型 Send-AOP 目标驱动完整 Gotham Pipeline Chain。
    async fn call_with_aop<Chain>(
        &self,
        mut state: State,
        chain: Chain,
        scope: Arc<WebRequestScope>,
        cancellation: CancellationToken,
        path_pattern: Arc<str>,
    ) -> HandlerResult
    where
        Chain: FnOnce(State) -> Pin<Box<HandlerFuture>> + Send + 'static,
    {
        if path_pattern.trim().is_empty() {
            return Ok(Self::aop_failure(
                state,
                &GothamAopError::MissingRouteMetadata,
            ));
        }

        // Gotham 已把标准 HTTP 元数据拆入 State。这里只复制 owned 元数据，不读取
        // 或缓冲请求 Body，也不让 State 借用跨越后续 await。
        let method = Method::borrow_from(&state).clone();
        let snapshot = HttpRequestSnapshot::from_parts(
            method.clone(),
            Uri::borrow_from(&state).clone(),
            *Version::borrow_from(&state),
            HeaderMap::borrow_from(&state).clone(),
        );
        let route = RouteMetadata::new(
            Arc::clone(&path_pattern),
            method.as_str().to_owned(),
            Arc::clone(&path_pattern),
        );
        let operation = route.aop_operation();
        let request_context = Arc::new(RequestContext::new(route, cancellation));
        request_context.extensions().insert(snapshot).await;
        state.put(VernalGothamRequestContext(Arc::clone(&request_context)));

        let Some(plan) = self.context.0.invocation_plans().get(&operation).cloned() else {
            return Ok(Self::aop_failure(
                state,
                &GothamAopError::invocation(InvocationError::PlanNotFound { operation }),
            ));
        };
        let invocation = HandlerInvocation::new(request_context, scope)
            .aop_invocation()
            .await;
        let envelope = Arc::new(GothamResponse::empty());
        let mut target = GothamBorrowedTarget::new(state, chain, Arc::clone(&envelope));
        let result = plan
            .invoke_borrowed(invocation, &mut target as &mut dyn BorrowedInvocationTarget)
            .await;

        match result {
            Ok(value) => {
                let Ok(returned) = value.downcast::<Arc<GothamResponse>>() else {
                    return Ok(Self::target_failure(
                        &mut target,
                        &GothamAopError::ResponseTypeMismatch,
                    ));
                };
                let Some(response) = returned.take().await else {
                    return Ok(Self::target_failure(
                        &mut target,
                        &GothamAopError::ResponseUnavailable,
                    ));
                };
                let state = Self::required_target_state(&mut target);
                Ok((state, response))
            }
            Err(error) => match error.into_target::<GothamUpstreamError>() {
                Ok(_) => Err(target.take_native_error().unwrap_or_else(|| {
                    let state = Self::required_target_state(&mut target);
                    (
                        state,
                        HandlerError::from(GothamAopError::ResponseUnavailable),
                    )
                })),
                Err(error) => Ok(Self::target_failure(
                    &mut target,
                    &GothamAopError::invocation(error),
                )),
            },
        }
    }

    /// 将严格 AOP 失败映射为 Gotham 原生响应。
    fn aop_failure(state: State, error: &GothamAopError) -> (State, Response<Body>) {
        let response = error.response(&state);
        (state, response)
    }

    /// 从借用型目标取回 State 后生成安全响应。
    fn target_failure<Chain>(
        target: &mut GothamBorrowedTarget<Chain>,
        error: &GothamAopError,
    ) -> (State, Response<Body>) {
        Self::aop_failure(Self::required_target_state(target), error)
    }

    /// 取回 Gotham Handler 合同保证返回的 State。
    fn required_target_state<Chain>(target: &mut GothamBorrowedTarget<Chain>) -> State {
        target.take_state().expect(
            "Gotham Handler contract must return State after the borrowed target is invoked",
        )
    }
}

impl NewMiddleware for VernalGothamMiddleware {
    type Instance = Self;

    fn new_middleware(&self) -> gotham::anyhow::Result<Self::Instance> {
        Ok(self.clone())
    }
}

impl Middleware for VernalGothamMiddleware {
    fn call<Chain>(self, mut state: State, chain: Chain) -> Pin<Box<HandlerFuture>>
    where
        Chain: FnOnce(State) -> Pin<Box<HandlerFuture>> + Send + 'static,
    {
        state.put(VernalGothamContext(Arc::clone(&self.context.0)));
        let scope = Arc::new(WebRequestScope::from_application_context(Arc::clone(
            &self.context.0,
        )));
        let cancellation = scope.cancellation().clone();
        state.put(VernalGothamRequestScope(Arc::clone(&scope)));

        // Handler Future 或响应 Body 被丢弃时，DropGuard 发出同步取消信号；
        // Tokio 清理任务负责执行异步关闭钩子。
        let cleanup_scope = Arc::clone(&scope);
        let cleanup_cancellation = cancellation.clone();
        tokio::spawn(async move {
            cleanup_cancellation.cancelled().await;
            let _ = cleanup_scope.close().await;
        });

        let request_guard = cancellation.clone().drop_guard();
        Box::pin(async move {
            let result = match &self.strict_aop_path_pattern {
                Some(path_pattern) => {
                    self.call_with_aop(
                        state,
                        chain,
                        Arc::clone(&scope),
                        cancellation,
                        Arc::clone(path_pattern),
                    )
                    .await
                }
                None => chain(state).await,
            };
            match result {
                Ok((state, response)) => {
                    let cancellation = request_guard.disarm();
                    let (parts, body) = response.into_parts();
                    let body = GothamScopedBody::new(body, scope, cancellation).boxed_unsync();
                    Ok((state, Response::from_parts(parts, body)))
                }
                Err((state, error)) => {
                    request_guard.disarm();
                    match scope.close().await {
                        Ok(()) => Err((state, error)),
                        Err(error) => Err((
                            state,
                            HandlerError::from(GothamRejection::scope_close(error)),
                        )),
                    }
                }
            }
        })
    }
}
