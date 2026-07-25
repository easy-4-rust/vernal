//! 条件组件评估诊断快照对象。

use serde::Serialize;

use crate::condition_contribution_counts::ConditionContributionCounts;

/// 一次条件模块构建期判断的只读、可序列化、脱敏结果。
///
/// 快照只包含静态模块名、条件类型名、命中状态、组件类型标识和各类声明数量；不会
/// 暴露属性键、属性值、期望值或底层错误正文。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConditionEvaluationSnapshot {
    module: String,
    condition: String,
    matched: bool,
    components: Vec<String>,
    trait_binding_count: usize,
    lifecycle_count: usize,
    event_listener_count: usize,
    application_runner_count: usize,
    scheduled_task_count: usize,
}

impl ConditionEvaluationSnapshot {
    /// 创建已经完成脱敏的条件评估快照。
    pub(crate) fn new(
        module: &'static str,
        condition: &'static str,
        matched: bool,
        components: Vec<String>,
        counts: ConditionContributionCounts,
    ) -> Self {
        Self {
            module: module.to_owned(),
            condition: condition.to_owned(),
            matched,
            components,
            trait_binding_count: counts.trait_bindings(),
            lifecycle_count: counts.lifecycles(),
            event_listener_count: counts.event_listeners(),
            application_runner_count: counts.application_runners(),
            scheduled_task_count: counts.scheduled_tasks(),
        }
    }

    /// 返回条件模块静态名称。
    #[must_use]
    pub fn module(&self) -> &str {
        &self.module
    }

    /// 返回稳定条件类型名。
    #[must_use]
    pub fn condition(&self) -> &str {
        &self.condition
    }

    /// 返回该模块是否进入最终依赖图。
    #[must_use]
    pub const fn matched(&self) -> bool {
        self.matched
    }

    /// 返回模块声明的组件类型标识。
    ///
    /// 未命中模块也保留类型标识，便于解释“为什么某组件没有进入容器”。
    #[must_use]
    pub fn components(&self) -> &[String] {
        &self.components
    }

    /// 返回模块声明的 Trait Binding 数量。
    #[must_use]
    pub const fn trait_binding_count(&self) -> usize {
        self.trait_binding_count
    }

    /// 返回模块声明的生命周期登记数量。
    #[must_use]
    pub const fn lifecycle_count(&self) -> usize {
        self.lifecycle_count
    }

    /// 返回模块声明的强类型事件监听器数量。
    #[must_use]
    pub const fn event_listener_count(&self) -> usize {
        self.event_listener_count
    }

    /// 返回模块声明的一次性应用 Runner 数量。
    #[must_use]
    pub const fn application_runner_count(&self) -> usize {
        self.application_runner_count
    }

    /// 返回模块声明的 Context 托管周期任务数量。
    #[must_use]
    pub const fn scheduled_task_count(&self) -> usize {
        self.scheduled_task_count
    }
}
