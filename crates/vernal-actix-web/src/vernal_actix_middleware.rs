//! Actix Web 中间件工厂对象。

use std::{future::Ready, rc::Rc, sync::Arc};

use actix_web::{
    Error,
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use vernal_context::ApplicationContext;

use crate::{ActixScopedBody, VernalActixService};

/// 为每个 Actix 请求注入 Context 并管理请求 Scope。
#[derive(Clone)]
pub struct VernalActixMiddleware {
    context: Arc<ApplicationContext>,
}

impl VernalActixMiddleware {
    /// 创建 Actix Web 中间件。
    #[must_use]
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self { context }
    }
}

impl<S, B> Transform<S, ServiceRequest> for VernalActixMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<ActixScopedBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = VernalActixService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(VernalActixService::new(
            Rc::new(service),
            Arc::clone(&self.context),
        )))
    }
}
