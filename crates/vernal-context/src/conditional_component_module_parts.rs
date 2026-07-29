//! 条件组件模块冻结贡献集合。

use vernal_beans::{ComponentDefinition, TraitBinding};

use crate::{
    application_runner_registrar::ApplicationRunnerRegistrar,
    event_listener_registrar::EventListenerRegistrar, lifecycle_registrar::LifecycleRegistrar,
    scheduled_task_registrar::ScheduledTaskRegistrar,
};

/// 保存一个条件模块等待按判断结果整体提交或排除的全部贡献。
///
/// 命名字段避免四元组随贡献种类增长而发生顺序错误；本对象只存在于应用构建阶段。
pub struct ConditionalComponentModuleParts {
    pub definitions: Vec<ComponentDefinition>,
    pub bindings: Vec<TraitBinding>,
    pub lifecycle_registrars: Vec<Box<LifecycleRegistrar>>,
    pub event_listener_registrars: Vec<Box<EventListenerRegistrar>>,
    pub application_runner_registrars: Vec<Box<ApplicationRunnerRegistrar>>,
    pub scheduled_task_registrars: Vec<Box<ScheduledTaskRegistrar>>,
}
