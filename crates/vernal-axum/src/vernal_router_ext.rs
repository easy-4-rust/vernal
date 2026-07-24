//! Axum Router 装配扩展对象。

use std::sync::Arc;

use axum::{Router, middleware};
use vernal_context::ApplicationContext;
use vernal_tower::VernalLayer;

use crate::AxumRequestScope;

/// 为 Axum `Router` 一次装配 Vernal Context 与请求作用域。
pub trait VernalRouterExt {
    /// 添加显式 Context 注入和请求 Scope Middleware。
    #[must_use]
    fn with_vernal(self, context: Arc<ApplicationContext>) -> Self;
}

impl<S> VernalRouterExt for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn with_vernal(self, context: Arc<ApplicationContext>) -> Self {
        self.layer(middleware::from_fn_with_state((), AxumRequestScope::handle))
            .layer(VernalLayer::new(context))
    }
}
