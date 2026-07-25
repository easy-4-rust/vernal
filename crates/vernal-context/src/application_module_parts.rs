//! 已暂存应用模块贡献集合。

use vernal_aop::Operation;
use vernal_ioc::{ComponentDefinition, TraitBinding};

use crate::{
    advisor_registration::AdvisorRegistration, lifecycle_registrar::LifecycleRegistrar,
    local_advisor_registration::LocalAdvisorRegistration,
    module_environment_contribution::ModuleEnvironmentContribution,
};

/// 保存已经完成模块配置、等待统一预检和提交的全部贡献。
///
/// 该对象只在 `register_module` 调用栈内存在，不进入运行期 Context。使用命名字段
/// 而不是大型元组，确保后续增加贡献种类时不会误换提交顺序。
pub(crate) struct ApplicationModuleParts {
    pub(crate) definitions: Vec<ComponentDefinition>,
    pub(crate) bindings: Vec<TraitBinding>,
    pub(crate) lifecycle_registrars: Vec<Box<LifecycleRegistrar>>,
    pub(crate) advisor_registrations: Vec<AdvisorRegistration>,
    pub(crate) local_advisor_registrations: Vec<LocalAdvisorRegistration>,
    pub(crate) operations: Vec<Operation>,
    pub(crate) environment_contributions: Vec<ModuleEnvironmentContribution>,
}

impl ApplicationModuleParts {
    /// 返回模块是否没有声明任何贡献。
    pub(crate) fn is_empty(&self) -> bool {
        self.definitions.is_empty()
            && self.bindings.is_empty()
            && self.lifecycle_registrars.is_empty()
            && self.advisor_registrations.is_empty()
            && self.local_advisor_registrations.is_empty()
            && self.operations.is_empty()
            && self.environment_contributions.is_empty()
    }
}
