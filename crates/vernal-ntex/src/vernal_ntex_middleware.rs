//! Ntex 中间件工厂对象。

use std::sync::Arc;

use ntex::service::Middleware;
use vernal_context::ApplicationContext;

use crate::VernalNtexService;

/// 为每个 Ntex 请求注入 Context 并管理请求 Scope。
#[derive(Clone)]
pub struct VernalNtexMiddleware {
    context: Arc<ApplicationContext>,
    strict_aop_path_pattern: Option<Arc<str>>,
}

impl VernalNtexMiddleware {
    /// 创建 Ntex 中间件工厂。
    #[must_use]
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self {
            context,
            strict_aop_path_pattern: None,
        }
    }

    /// 创建注入 Context、管理 Scope 并执行严格 Local-AOP 的中间件。
    ///
    /// Ntex 2 的公共请求 API 不暴露匹配后的 `ResourceDef`。调用方必须把该
    /// 中间件安装到具体 `web::resource(...).wrap(...)`，并传入同一个完整、
    /// 低基数路径模板；HTTP Method 仍从真实请求读取。缺少预编译计划时
    /// fail-closed，不回退到用户输入的原始路径。
    #[must_use]
    pub fn strict_aop(context: Arc<ApplicationContext>, path_pattern: impl Into<Arc<str>>) -> Self {
        Self {
            context,
            strict_aop_path_pattern: Some(path_pattern.into()),
        }
    }
}

impl<S> Middleware<S> for VernalNtexMiddleware {
    type Service = VernalNtexService<S>;

    fn create(&self, service: S) -> Self::Service {
        VernalNtexService::new(
            service,
            Arc::clone(&self.context),
            self.strict_aop_path_pattern.clone(),
        )
    }
}
