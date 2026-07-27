//! Tonic AOP Service 对象。

use std::{
    error::Error,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use http::{Request, Response};
use tonic::body::BoxBody;
use tower::{Layer, Service};
use vernal_tower::{AopLayer, AopService, AopServiceError, MissingPlanPolicy};

use crate::{TonicAopErrorMapper, TonicRouteResolver};

/// 在 gRPC 协议边界执行共享 AOP Service，并把策略失败写成 gRPC Status 响应。
///
/// 下游真实传输错误仍作为 `Service::Error` 返回；认证、授权、取消和计划错误则
/// 生成带 `grpc-status` 的原生空 Body 响应，客户端不会观察到连接级失败。
#[derive(Clone)]
pub struct TonicAopService<S> {
    inner: AopService<S, TonicRouteResolver>,
}

impl<S> TonicAopService<S> {
    /// 使用共享 Tower AOP Layer 包装 Tonic 服务。
    pub(crate) fn new(inner: S, missing_plan_policy: MissingPlanPolicy) -> Self {
        Self {
            inner: AopLayer::new(TonicRouteResolver)
                .with_missing_plan_policy(missing_plan_policy)
                .layer(inner),
        }
    }
}

impl<S, B> Service<Request<B>> for TonicAopService<S>
where
    S: Service<Request<B>, Response = Response<BoxBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Error + Send + Sync + 'static,
    B: Send + 'static,
{
    type Response = Response<BoxBody>;
    type Error = AopServiceError<S::Error>;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, context: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(context)
    }

    fn call(&mut self, request: Request<B>) -> Self::Future {
        let future = self.inner.call(request);
        Box::pin(async move {
            match future.await {
                Ok(response) => Ok(response),
                Err(error @ AopServiceError::Upstream(_)) => Err(error),
                Err(error) => Ok(TonicAopErrorMapper::from_service_error(&error).into_http()),
            }
        })
    }
}
