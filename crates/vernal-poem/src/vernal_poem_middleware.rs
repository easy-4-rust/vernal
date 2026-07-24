//! Poem 中间件工厂对象。

use std::sync::Arc;

use poem::{Endpoint, Middleware};
use vernal_context::ApplicationContext;

use crate::VernalPoemEndpoint;

/// 为每个 Poem 请求注入 Context 并管理请求 Scope。
#[derive(Clone)]
pub struct VernalPoemMiddleware {
    context: Arc<ApplicationContext>,
}

impl VernalPoemMiddleware {
    /// 创建 Poem 中间件。
    #[must_use]
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self { context }
    }
}

impl<E> Middleware<E> for VernalPoemMiddleware
where
    E: Endpoint,
{
    type Output = VernalPoemEndpoint<E>;

    fn transform(&self, endpoint: E) -> Self::Output {
        VernalPoemEndpoint::new(endpoint, Arc::clone(&self.context))
    }
}
