//! Actix Web 请求生命周期 Service 对象。

use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll},
};

use actix_web::{
    Error, HttpMessage,
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse},
};
use tokio_util::sync::CancellationToken;
use vernal_context::ApplicationContext;
use vernal_web::WebRequestScope;

use crate::{ActixRejection, ActixScopedBody};

type ServiceFuture<B> =
    Pin<Box<dyn Future<Output = Result<ServiceResponse<ActixScopedBody<B>>, Error>>>>;

/// 执行 Actix 请求 Context 注入、Scope 创建和响应 Body 包装。
pub struct VernalActixService<S> {
    service: Rc<S>,
    context: Arc<ApplicationContext>,
}

impl<S> VernalActixService<S> {
    /// 由中间件工厂创建请求 Service。
    pub(crate) fn new(service: Rc<S>, context: Arc<ApplicationContext>) -> Self {
        Self { service, context }
    }
}

impl<S, B> Service<ServiceRequest> for VernalActixService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<ActixScopedBody<B>>;
    type Error = Error;
    type Future = ServiceFuture<B>;

    fn poll_ready(&self, context: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(context)
    }

    fn call(&self, request: ServiceRequest) -> Self::Future {
        request.extensions_mut().insert(Arc::clone(&self.context));
        let cancellation = CancellationToken::new();
        let scope = Arc::new(WebRequestScope::new(cancellation.clone()));
        request.extensions_mut().insert(Arc::clone(&scope));

        // 请求 Future 或响应 Body 被丢弃时，由 DropGuard 只发出同步取消信号，
        // 这里预先启动的任务负责执行真正的异步关闭。
        let cleanup_scope = Arc::clone(&scope);
        let cleanup_cancellation = cancellation.clone();
        actix_web::rt::spawn(async move {
            cleanup_cancellation.cancelled().await;
            let _ = cleanup_scope.close().await;
        });

        let request_guard = cancellation.drop_guard();
        let service = Rc::clone(&self.service);
        Box::pin(async move {
            match service.call(request).await {
                Ok(response) => {
                    let cancellation = request_guard.disarm();
                    Ok(
                        response
                            .map_body(|_, body| ActixScopedBody::new(body, scope, cancellation)),
                    )
                }
                Err(error) => {
                    request_guard.disarm();
                    scope.close().await.map_err(ActixRejection::scope_close)?;
                    Err(error)
                }
            }
        })
    }
}
