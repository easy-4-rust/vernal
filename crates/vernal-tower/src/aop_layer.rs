//! Tower AOP Layer 对象。

use tower::Layer;

use crate::{AopService, ExtensionRouteResolver, MissingPlanPolicy};

/// 把 Vernal 预编译调用计划织入任意 Tower `Service`。
///
/// 推荐层次为
/// `VernalLayer -> RequestScopeLayer -> ContextPropagationLayer -> AopLayer -> Handler`。
/// 上层框架适配器只负责提供路由元数据，AOP 的顺序、短路、取消与错误语义在
/// 此处统一。
#[derive(Clone)]
pub struct AopLayer<R = ExtensionRouteResolver> {
    resolver: R,
    missing_plan_policy: MissingPlanPolicy,
}

impl AopLayer<ExtensionRouteResolver> {
    /// 创建从请求扩展读取 [`vernal_web::RouteMetadata`] 的严格 AOP Layer。
    #[must_use]
    pub fn from_extension() -> Self {
        Self::new(ExtensionRouteResolver)
    }
}

impl<R> AopLayer<R> {
    /// 使用指定路由解析器创建严格 AOP Layer。
    ///
    /// 默认缺失计划时拒绝请求，防止安全拦截器因配置遗漏而被静默绕过。
    #[must_use]
    pub const fn new(resolver: R) -> Self {
        Self {
            resolver,
            missing_plan_policy: MissingPlanPolicy::Reject,
        }
    }

    /// 设置路由缺少 AOP 调用计划时的行为。
    #[must_use]
    pub const fn with_missing_plan_policy(mut self, policy: MissingPlanPolicy) -> Self {
        self.missing_plan_policy = policy;
        self
    }
}

impl<S, R> Layer<S> for AopLayer<R>
where
    R: Clone,
{
    type Service = AopService<S, R>;

    fn layer(&self, inner: S) -> Self::Service {
        AopService::new(inner, self.resolver.clone(), self.missing_plan_policy)
    }
}
