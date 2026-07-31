//! 已暂存应用模块贡献集合。

use vernal_aop::Operation;
use vernal_beans::{ComponentDefinition, TraitBinding};

use crate::{
    ConditionalComponentModule, advisor_registration::AdvisorRegistration,
    application_runner_registrar::ApplicationRunnerRegistrar,
    event_listener_registrar::EventListenerRegistrar, lifecycle_registrar::LifecycleRegistrar,
    local_advisor_registration::LocalAdvisorRegistration,
    module_environment_contribution::ModuleEnvironmentContribution,
    scheduled_task_registrar::ScheduledTaskRegistrar,
};

/// 保存已经完成模块配置、等待统一预检和提交的全部贡献。
///
/// 该对象只在 `register_module` 调用栈内存在，不进入运行期 Context。使用命名字段
/// 而不是大型元组，确保后续增加贡献种类时不会误换提交顺序。
pub struct ApplicationModuleParts {
    /// 组件定义集合。
    pub definitions: Vec<ComponentDefinition>,
    /// Trait 绑定集合。
    pub bindings: Vec<TraitBinding>,
    /// 生命周期登记器集合。
    pub lifecycle_registrars: Vec<Box<LifecycleRegistrar>>,
    /// 事件监听器登记器集合。
    pub event_listener_registrars: Vec<Box<EventListenerRegistrar>>,
    /// 应用 Runner 登记器集合。
    pub application_runner_registrars: Vec<Box<ApplicationRunnerRegistrar>>,
    /// 周期任务登记器集合。
    pub scheduled_task_registrars: Vec<Box<ScheduledTaskRegistrar>>,
    /// 切面顾问登记集合。
    pub advisor_registrations: Vec<AdvisorRegistration>,
    /// 本地切面顾问登记集合。
    pub local_advisor_registrations: Vec<LocalAdvisorRegistration>,
    /// 切面操作集合。
    pub operations: Vec<Operation>,
    /// 环境贡献集合。
    pub environment_contributions: Vec<ModuleEnvironmentContribution>,
    /// 条件组件模块集合。
    pub conditional_modules: Vec<ConditionalComponentModule>,
}

impl ApplicationModuleParts {
    /// 返回模块是否没有声明任何贡献。
    pub const fn is_empty(&self) -> bool {
        self.definitions.is_empty()
            && self.bindings.is_empty()
            && self.lifecycle_registrars.is_empty()
            && self.event_listener_registrars.is_empty()
            && self.application_runner_registrars.is_empty()
            && self.scheduled_task_registrars.is_empty()
            && self.advisor_registrations.is_empty()
            && self.local_advisor_registrations.is_empty()
            && self.operations.is_empty()
            && self.environment_contributions.is_empty()
            && self.conditional_modules.is_empty()
    }
}
