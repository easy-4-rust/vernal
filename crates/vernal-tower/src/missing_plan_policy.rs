//! Web AOP 缺失调用计划策略对象。

/// 当路由没有预编译 AOP 调用计划时采用的行为。
///
/// 默认拒绝请求以避免鉴权切面因配置遗漏而被静默绕过。只有明确不要求切面的
/// 路由才应选择 [`MissingPlanPolicy::Proceed`]。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MissingPlanPolicy {
    /// 返回结构化的 `PlanNotFound` 错误，不执行下游 Handler。
    #[default]
    Reject,
    /// 不经过 AOP 调用链，直接执行下游 Handler。
    Proceed,
}
