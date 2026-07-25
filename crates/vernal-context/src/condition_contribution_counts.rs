//! 条件模块贡献数量值对象。

/// 聚合条件模块五类非组件贡献的脱敏数量。
///
/// 使用命名字段代替位置数组，避免新增贡献种类时把诊断计数写错槽位。该对象只在
/// 构建期快照创建过程中传递，不进入公开 API 或运行期状态。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ConditionContributionCounts {
    trait_bindings: usize,
    lifecycles: usize,
    event_listeners: usize,
    application_runners: usize,
    scheduled_tasks: usize,
}

impl ConditionContributionCounts {
    /// 创建一组命名贡献计数。
    pub(crate) const fn new(
        trait_bindings: usize,
        lifecycles: usize,
        event_listeners: usize,
        application_runners: usize,
        scheduled_tasks: usize,
    ) -> Self {
        Self {
            trait_bindings,
            lifecycles,
            event_listeners,
            application_runners,
            scheduled_tasks,
        }
    }

    /// 返回 Trait Binding 数量。
    pub(crate) const fn trait_bindings(self) -> usize {
        self.trait_bindings
    }

    /// 返回生命周期声明数量。
    pub(crate) const fn lifecycles(self) -> usize {
        self.lifecycles
    }

    /// 返回事件监听声明数量。
    pub(crate) const fn event_listeners(self) -> usize {
        self.event_listeners
    }

    /// 返回应用 Runner 声明数量。
    pub(crate) const fn application_runners(self) -> usize {
        self.application_runners
    }

    /// 返回周期任务声明数量。
    pub(crate) const fn scheduled_tasks(self) -> usize {
        self.scheduled_tasks
    }
}
