//! Tower 请求上下文传播 Service 对象。

use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use http::Request;
use tower::Service;
use vernal_http::HttpRequestSnapshot;
use vernal_web::{RequestContext, WebRequestScope};

use crate::{ContextPropagationError, TowerRouteResolver};

/// 把 Tower 原生请求投影为 Vernal 框架中立请求上下文。
///
/// Service 不读取或替换 Body，只捕获 Method、URI、Version 与 Header 的 owned
/// 快照。若具体框架已经提供更精确的 `RequestContext` 或快照，则保留原对象，
/// 使嵌套路由、反向代理和框架专有 original-uri 语义不会被通用层覆盖。
#[derive(Clone)]
pub struct ContextPropagationService<S, R> {
    inner: S,
    resolver: R,
}

impl<S, R> ContextPropagationService<S, R> {
    /// 由 `ContextPropagationLayer` 创建传播 Service。
    pub(crate) const fn new(inner: S, resolver: R) -> Self {
        Self { inner, resolver }
    }
}

impl<S, R, B> Service<Request<B>> for ContextPropagationService<S, R>
where
    S: Service<Request<B>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
    R: TowerRouteResolver<B>,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = ContextPropagationError<S::Error>;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, context: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner
            .poll_ready(context)
            .map_err(ContextPropagationError::Upstream)
    }

    fn call(&mut self, mut request: Request<B>) -> Self::Future {
        // Scope 是请求取消和异步清理的唯一所有者。传播层不创建第二个令牌，
        // 从而保证 Handler、AOP 与 Body Drop 观察到同一取消状态。
        let Some(scope) = request.extensions().get::<Arc<WebRequestScope>>().cloned() else {
            return Box::pin(async { Err(ContextPropagationError::MissingRequestScope) });
        };

        // 框架 Adapter 可以预先写入包含 original-uri、匹配模板或 deadline 的
        // 上下文；通用层只在缺失时才使用路由解析器构造，避免降低元数据精度。
        let request_context =
            if let Some(context) = request.extensions().get::<Arc<RequestContext>>().cloned() {
                context
            } else {
                let Some(route) = self.resolver.resolve(&request) else {
                    return Box::pin(async { Err(ContextPropagationError::MissingRouteMetadata) });
                };
                let context = Arc::new(RequestContext::new(route, scope.cancellation().clone()));
                request.extensions_mut().insert(Arc::clone(&context));
                context
            };

        let snapshot = HttpRequestSnapshot::capture(&request);
        request
            .extensions_mut()
            .insert(request_context.cancellation().clone());

        // Tower 要求 call 使用刚通过 poll_ready 的实例；替换字段后把 ready Service
        // 和 owned Request 一起移入 Future，不在共享锁中等待下游。
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        Box::pin(async move {
            if !request_context
                .extensions()
                .contains::<HttpRequestSnapshot>()
                .await
            {
                request_context.extensions().insert(snapshot).await;
            }
            inner
                .call(request)
                .await
                .map_err(ContextPropagationError::Upstream)
        })
    }
}
