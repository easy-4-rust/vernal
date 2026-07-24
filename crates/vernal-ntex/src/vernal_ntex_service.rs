//! Ntex 请求生命周期 Service 对象。

use std::sync::Arc;

use ntex::{
    http::body::{Body, ResponseBody},
    service::{Service, ServiceCtx},
    web::{ErrorRenderer, WebRequest, WebResponse},
};
use tokio_util::sync::CancellationToken;
use vernal_context::ApplicationContext;
use vernal_web::WebRequestScope;

use crate::{NtexRejection, NtexScopedBody};

/// 执行 Ntex 请求 Context 注入、Scope 创建和响应 Body 包装。
pub struct VernalNtexService<S> {
    service: S,
    context: Arc<ApplicationContext>,
}

impl<S> VernalNtexService<S> {
    /// 由中间件工厂创建请求 Service。
    pub(crate) fn new(service: S, context: Arc<ApplicationContext>) -> Self {
        Self { service, context }
    }
}

impl<S, Err> Service<WebRequest<Err>> for VernalNtexService<S>
where
    S: Service<WebRequest<Err>, Response = WebResponse>,
    S::Error: From<NtexRejection>,
    Err: ErrorRenderer,
{
    type Response = WebResponse;
    type Error = S::Error;

    async fn call(
        &self,
        request: WebRequest<Err>,
        service_context: ServiceCtx<'_, Self>,
    ) -> Result<Self::Response, Self::Error> {
        request.extensions_mut().insert(Arc::clone(&self.context));
        let cancellation = CancellationToken::new();
        let scope = Arc::new(WebRequestScope::new(cancellation.clone()));
        request.extensions_mut().insert(Arc::clone(&scope));

        // Body 或请求 Future 被丢弃时，DropGuard 只能同步发出取消信号；
        // 预启动的本地任务负责执行真正的异步关闭钩子。
        let cleanup_scope = Arc::clone(&scope);
        let cleanup_cancellation = cancellation.clone();
        ntex::rt::spawn(async move {
            cleanup_cancellation.cancelled().await;
            let _ = cleanup_scope.close().await;
        });

        let request_guard = cancellation.drop_guard();
        match service_context.call(&self.service, request).await {
            Ok(response) => {
                let cancellation = request_guard.disarm();
                Ok(response.map_body(move |_head, body| {
                    ResponseBody::Other(Body::from_message(NtexScopedBody::new(
                        body,
                        scope,
                        cancellation,
                    )))
                }))
            }
            Err(error) => {
                request_guard.disarm();
                scope.close().await.map_err(NtexRejection::scope_close)?;
                Err(error)
            }
        }
    }
}
