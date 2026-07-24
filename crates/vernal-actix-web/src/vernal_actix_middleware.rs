//! Actix Web 中间件工厂对象。

use std::{future::Ready, rc::Rc, sync::Arc};

use actix_web::{
    Error,
    body::{EitherBody, MessageBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use vernal_context::ApplicationContext;

use crate::{ActixScopedBody, VernalActixService};

/// 为每个 Actix 请求注入 Context 并管理请求 Scope。
#[derive(Clone)]
pub struct VernalActixMiddleware {
    context: Arc<ApplicationContext>,
    strict_aop: bool,
}

impl VernalActixMiddleware {
    /// 创建 Actix Web 中间件。
    #[must_use]
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self {
            context,
            strict_aop: false,
        }
    }

    /// 创建注入 Context、管理 Scope 并执行严格 Local-AOP 的中间件。
    ///
    /// 中间件应通过 `web::resource(...).wrap(...)` 包裹已经声明的具体资源，
    /// 使用 Actix 匹配后的低基数 Resource Pattern 构建 Operation；缺少路由
    /// 元数据或预编译计划时 fail-closed，不回退到用户输入的原始路径。
    #[must_use]
    pub fn strict_aop(context: Arc<ApplicationContext>) -> Self {
        Self {
            context,
            strict_aop: true,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for VernalActixMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<ActixScopedBody<EitherBody<B>>>;
    type Error = Error;
    type InitError = ();
    type Transform = VernalActixService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(VernalActixService::new(
            Rc::new(service),
            Arc::clone(&self.context),
            self.strict_aop,
        )))
    }
}
