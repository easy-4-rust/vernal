//! Tonic AOP Layer 对象。

use tower::Layer;
use vernal_tower::MissingPlanPolicy;

use crate::TonicAopService;

/// 为 Tonic 服务装配严格 Vernal AOP 与 gRPC Status 映射。
#[derive(Clone, Copy, Debug)]
pub struct TonicAopLayer {
    missing_plan_policy: MissingPlanPolicy,
}

impl TonicAopLayer {
    /// 创建缺失调用计划时 fail-closed 的 Tonic AOP Layer。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            missing_plan_policy: MissingPlanPolicy::Reject,
        }
    }

    /// 显式设置缺失调用计划策略。
    #[must_use]
    pub const fn with_missing_plan_policy(mut self, policy: MissingPlanPolicy) -> Self {
        self.missing_plan_policy = policy;
        self
    }
}

impl Default for TonicAopLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for TonicAopLayer {
    type Service = TonicAopService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TonicAopService::new(inner, self.missing_plan_policy)
    }
}
