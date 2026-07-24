//! Ntex 中间件工厂对象。

use std::sync::Arc;

use ntex::service::Middleware;
use vernal_context::ApplicationContext;

use crate::VernalNtexService;

/// 为每个 Ntex 请求注入 Context 并管理请求 Scope。
#[derive(Clone)]
pub struct VernalNtexMiddleware {
    context: Arc<ApplicationContext>,
}

impl VernalNtexMiddleware {
    /// 创建 Ntex 中间件工厂。
    #[must_use]
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self { context }
    }
}

impl<S> Middleware<S> for VernalNtexMiddleware {
    type Service = VernalNtexService<S>;

    fn create(&self, service: S) -> Self::Service {
        VernalNtexService::new(service, Arc::clone(&self.context))
    }
}
