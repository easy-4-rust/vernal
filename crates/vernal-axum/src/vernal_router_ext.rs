//! Axum Router 装配扩展对象。

use std::sync::Arc;

use axum::{Router, middleware};
use tower::ServiceBuilder;
use vernal_context::ApplicationContext;
use vernal_tower::{AopLayer, ErrorMappingLayer, VernalLayer};

use crate::{AxumRequestScope, AxumRouteResolver, axum_aop_error_mapper::AxumAopErrorMapper};

/// 为 Axum `Router` 一次装配 Vernal Context 与请求作用域。
pub trait VernalRouterExt {
    /// 添加显式 Context 注入和请求 Scope Middleware。
    #[must_use]
    fn with_vernal(self, context: Arc<ApplicationContext>) -> Self;

    /// 添加 Context、请求 Scope 与严格 AOP 调用链。
    ///
    /// 路由没有预编译调用计划时返回安全的 500 响应，不会执行 Handler。应用应
    /// 在 [`vernal_context::VernalApplicationBuilder`] 中声明对应的
    /// `Operation(path_template, http_method)`。
    #[must_use]
    fn with_vernal_aop(self, context: Arc<ApplicationContext>) -> Self;
}

impl<S> VernalRouterExt for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn with_vernal(self, context: Arc<ApplicationContext>) -> Self {
        self.layer(middleware::from_fn_with_state((), AxumRequestScope::handle))
            .layer(VernalLayer::new(context))
    }

    fn with_vernal_aop(self, context: Arc<ApplicationContext>) -> Self {
        self.layer(
            ServiceBuilder::new()
                .layer(ErrorMappingLayer::new(AxumAopErrorMapper))
                .layer(AopLayer::new(AxumRouteResolver)),
        )
        .layer(middleware::from_fn_with_state((), AxumRequestScope::handle))
        .layer(VernalLayer::new(context))
    }
}
