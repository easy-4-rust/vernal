//! Poem 中间件工厂对象。

use std::sync::Arc;

use poem::{Endpoint, Middleware};
use vernal_context::ApplicationContext;

use crate::VernalPoemEndpoint;

/// 为每个 Poem 请求注入 Context 并管理请求 Scope。
#[derive(Clone)]
pub struct VernalPoemMiddleware {
    context: Arc<ApplicationContext>,
    strict_aop: bool,
}

impl VernalPoemMiddleware {
    /// 创建 Poem 中间件。
    #[must_use]
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self {
            context,
            strict_aop: false,
        }
    }

    /// 创建同时注入 Context、管理请求 Scope 并执行严格 AOP 的中间件。
    ///
    /// 该中间件必须包裹注册到 `Route::at` 的具体 Endpoint，而不是包裹整个
    /// `Route`；Poem 只有完成路由匹配后才会把低基数 `PathPattern` 写入 Request。
    /// 缺少 `PathPattern` 时严格模式 fail-closed，不会回退到含用户输入的原始 URI。
    #[must_use]
    pub fn strict_aop(context: Arc<ApplicationContext>) -> Self {
        Self {
            context,
            strict_aop: true,
        }
    }
}

impl<E> Middleware<E> for VernalPoemMiddleware
where
    E: Endpoint + 'static,
{
    type Output = VernalPoemEndpoint<E>;

    fn transform(&self, endpoint: E) -> Self::Output {
        VernalPoemEndpoint::new(endpoint, Arc::clone(&self.context), self.strict_aop)
    }
}
