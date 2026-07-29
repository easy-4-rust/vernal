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
    pub definitions: Vec<ComponentDefinition>,
    pub bindings: Vec<TraitBinding>,
    pub lifecycle_registrars: Vec<Box<LifecycleRegistrar>>,
    pub event_listener_registrars: Vec<Box<EventListenerRegistrar>>,
    pub application_runner_registrars: Vec<Box<ApplicationRunnerRegistrar>>,
    pub scheduled_task_registrars: Vec<Box<ScheduledTaskRegistrar>>,
    pub advisor_registrations: Vec<AdvisorRegistration>,
    pub local_advisor_registrations: Vec<LocalAdvisorRegistration>,
    pub operations: Vec<Operation>,
    pub environment_contributions: Vec<ModuleEnvironmentContribution>,
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
