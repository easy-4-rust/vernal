#![forbid(unsafe_code)]
#![doc = "Vernal 的 Tokio-first 类型驱动控制反转内核。"]

mod build_plan;
mod component_contract;
mod component_definition;
mod component_key;
mod component_registry;
mod component_scope;
mod container;
mod definition_error;
mod dependency;
mod graph_error;
mod graph_planner;
mod qualifier;
mod registry_builder;
mod resolve_error;
mod resolver;

pub use build_plan::BuildPlan;
pub use component_contract::Component;
pub use component_definition::ComponentDefinition;
pub use component_key::ComponentKey;
pub use component_registry::Registry;
pub use component_scope::Scope;
pub use container::Container;
pub use definition_error::DefinitionError;
pub use dependency::Dependency;
pub use graph_error::GraphError;
pub use qualifier::Qualifier;
pub use registry_builder::RegistryBuilder;
pub use resolve_error::ResolveError;
pub use resolver::Resolver;

/// 返回当前 `IoC` 内核的成熟度状态。
#[must_use]
pub const fn project_status() -> &'static str {
    vernal_core::PROJECT_STATUS
}
