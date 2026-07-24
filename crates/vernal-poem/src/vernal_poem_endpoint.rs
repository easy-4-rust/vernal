//! Poem 请求生命周期与严格 AOP Endpoint 对象。

use std::{any::type_name, sync::Arc};

use poem::{Body, Endpoint, IntoResponse, PathPattern, Request, Response, Result};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use vernal_aop::{InvocationError, InvocationTarget, InvocationValue};
use vernal_context::ApplicationContext;
use vernal_http::HttpRequestSnapshot;
use vernal_web::{HandlerInvocation, RequestContext, RouteMetadata, WebRequestScope};

use crate::{PoemAopError, PoemRejection, PoemResponse, PoemScopedStream};

/// 执行 Poem Context 注入、请求 Scope 生命周期与可选严格 AOP。
///
/// Endpoint 使用 `Arc<E>` 保存 Poem 原生目标，使严格模式可以把完整 Handler
/// Future 放入 Vernal Around 调用链；旧的非 AOP 模式仍沿用同一生命周期实现。
pub struct VernalPoemEndpoint<E> {
    endpoint: Arc<E>,
    context: Arc<ApplicationContext>,
    strict_aop: bool,
}

impl<E: 'static> VernalPoemEndpoint<E> {
    /// 由中间件工厂创建请求 Endpoint。
    pub(crate) fn new(endpoint: E, context: Arc<ApplicationContext>, strict_aop: bool) -> Self {
        Self {
            endpoint: Arc::new(endpoint),
            context,
            strict_aop,
        }
    }

    /// 以低基数 Poem 路由模板执行预编译 AOP 计划。
    async fn call_with_aop(
        &self,
        mut request: Request,
        scope: Arc<WebRequestScope>,
        cancellation: CancellationToken,
    ) -> Result<Response>
    where
        E: Endpoint,
    {
        let path_pattern = request
            .data::<PathPattern>()
            .cloned()
            .ok_or(PoemAopError::MissingRouteMetadata)?;
        let method: Arc<str> = request.method().as_str().into();
        let route = RouteMetadata::new(
            Arc::clone(&path_pattern.0),
            Arc::clone(&method),
            Arc::clone(&path_pattern.0),
        );
        let operation = route.aop_operation();
        // Poem 的真实 Server 入口会保存 original_uri；它的测试 RequestBuilder
        // 不会初始化该字段。仅当 original_uri 仍是默认根路径且当前 URI 明确不是
        // 根路径时回退到当前 URI，既保留 nest 场景的外部地址，也支持原生测试。
        let snapshot_uri = if request.original_uri().path() == "/" && request.uri().path() != "/" {
            request.uri().clone()
        } else {
            request.original_uri().clone()
        };
        let snapshot = HttpRequestSnapshot::from_parts(
            request.method().clone(),
            snapshot_uri,
            request.version(),
            request.headers().clone(),
        );
        let request_context = Arc::new(RequestContext::new(route, cancellation));
        request_context.extensions().insert(snapshot).await;
        request
            .extensions_mut()
            .insert(Arc::clone(&request_context));

        let invocation = HandlerInvocation::new(request_context, scope)
            .aop_invocation()
            .await;
        let plan = self
            .context
            .invocation_plans()
            .get(&operation)
            .ok_or_else(|| {
                PoemAopError::invocation(InvocationError::PlanNotFound {
                    operation: operation.clone(),
                })
            })?;

        // Request 只能交给原生 Endpoint 一次。Tokio Mutex 让一次性移交信封满足
        // InvocationTarget 的 Send + Sync 合同，而不复制或借用 Poem Request。
        let request = Arc::new(Mutex::new(Some(request)));
        let endpoint = Arc::clone(&self.endpoint);
        let target: Arc<InvocationTarget> = Arc::new(move |_invocation| {
            let request = Arc::clone(&request);
            let endpoint = Arc::clone(&endpoint);
            Box::pin(async move {
                let request =
                    request.lock().await.take().ok_or_else(|| {
                        InvocationError::target(PoemAopError::RequestAlreadyTaken)
                    })?;
                let output = endpoint
                    .call(request)
                    .await
                    .map_err(InvocationError::target)?;
                Ok(Box::new(PoemResponse::new(output.into_response())) as InvocationValue)
            })
        });

        let value = match plan.invoke(invocation, target).await {
            Ok(value) => value,
            Err(InvocationError::Target { source }) => {
                // 原生 Endpoint 错误必须保留 Poem 自己的状态码与响应语义；只有
                // 非 Poem 目标错误（如 WebFailure）才进入 Vernal AOP 映射。
                match source.downcast::<poem::Error>() {
                    Ok(error) => return Err(*error),
                    Err(source) => {
                        return Err(
                            PoemAopError::invocation(InvocationError::Target { source }).into()
                        );
                    }
                }
            }
            Err(error) => return Err(PoemAopError::invocation(error).into()),
        };
        let response = value
            .downcast::<PoemResponse>()
            .map_err(|_| {
                PoemAopError::invocation(InvocationError::ReturnTypeMismatch {
                    expected: type_name::<PoemResponse>(),
                })
            })?
            .take()
            .await
            .ok_or(PoemAopError::ResponseAlreadyTaken)?;
        Ok(response)
    }
}

impl<E> Endpoint for VernalPoemEndpoint<E>
where
    E: Endpoint + 'static,
{
    type Output = Response;

    async fn call(&self, mut request: Request) -> Result<Self::Output> {
        request.extensions_mut().insert(Arc::clone(&self.context));
        let scope = Arc::new(WebRequestScope::from_application_context(Arc::clone(
            &self.context,
        )));
        let cancellation = scope.cancellation().clone();
        request.extensions_mut().insert(Arc::clone(&scope));

        // 请求 Future 或响应流被丢弃时，由 DropGuard 只发出同步取消信号；
        // 这里预先启动的 Tokio 任务负责执行真正的异步 Scope 关闭。
        let cleanup_scope = Arc::clone(&scope);
        let cleanup_cancellation = cancellation.clone();
        tokio::spawn(async move {
            cleanup_cancellation.cancelled().await;
            let _ = cleanup_scope.close().await;
        });

        let request_guard = cancellation.clone().drop_guard();
        let response = if self.strict_aop {
            self.call_with_aop(request, Arc::clone(&scope), cancellation.clone())
                .await
        } else {
            self.endpoint
                .call(request)
                .await
                .map(IntoResponse::into_response)
        };

        match response {
            Ok(mut response) => {
                let cancellation = request_guard.disarm();
                let body_stream = response.take_body().into_bytes_stream();
                response.set_body(Body::from_bytes_stream(PoemScopedStream::new(
                    body_stream,
                    scope,
                    cancellation,
                )));
                Ok(response)
            }
            Err(error) => {
                request_guard.disarm();
                scope.close().await.map_err(PoemRejection::scope_close)?;
                Err(error)
            }
        }
    }
}
