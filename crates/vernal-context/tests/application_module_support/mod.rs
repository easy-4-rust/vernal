//! 应用模块合同测试对象集合。

mod commerce_module;
mod conditional_application_module;
mod definition_conflict_module;
mod empty_module;
mod environment_conflict_module;
mod failing_module;
mod invalid_module;
mod module_probe;
mod module_service;

pub use commerce_module::CommerceModule;
pub use conditional_application_module::ConditionalApplicationModule;
pub use definition_conflict_module::DefinitionConflictModule;
pub use empty_module::EmptyModule;
pub use environment_conflict_module::EnvironmentConflictModule;
pub use failing_module::FailingModule;
pub use invalid_module::InvalidModule;
pub use module_probe::ModuleProbe;
pub use module_service::ModuleService;
