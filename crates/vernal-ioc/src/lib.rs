#![forbid(unsafe_code)]
#![doc = "Vernal 的 Tokio-first 类型驱动控制反转内核。"]

mod build_plan;
mod component_contract;
mod component_definition;
mod component_key;
mod component_registry;
mod component_scope;
mod component_snapshot;
mod container;
mod definition_error;
mod dependency;
mod graph_error;
mod graph_planner;
mod qualifier;
mod registry_builder;
mod registry_snapshot;
mod registry_summary;
mod resolution_tracker;
mod resolve_error;
mod resolver;
mod scope_close_failure;
mod scope_context;
mod scope_error;
mod scope_future;
mod scope_key;
mod scope_operation_guard;
mod scope_runtime_state;
mod scope_state;
mod trait_binding;
mod trait_binding_snapshot;
mod trait_key;

pub use build_plan::BuildPlan;
pub use component_contract::Component;
pub use component_definition::ComponentDefinition;
pub use component_key::ComponentKey;
pub use component_registry::Registry;
pub use component_scope::Scope;
pub use component_snapshot::ComponentSnapshot;
pub use container::Container;
pub use definition_error::DefinitionError;
pub use dependency::Dependency;
pub use graph_error::GraphError;
pub use qualifier::Qualifier;
pub use registry_builder::RegistryBuilder;
pub use registry_snapshot::RegistrySnapshot;
pub use registry_summary::RegistrySummary;
pub use resolve_error::ResolveError;
pub use resolver::Resolver;
pub use scope_context::ScopeContext;
pub use scope_error::ScopeError;
pub use scope_future::ScopeFuture;
pub use scope_key::ScopeKey;
pub use scope_state::ScopeState;
pub use trait_binding::TraitBinding;
pub use trait_binding_snapshot::TraitBindingSnapshot;
pub use trait_key::TraitKey;

/// 返回当前 `IoC` 内核的成熟度状态。
#[must_use]
pub const fn project_status() -> &'static str {
    vernal_core::PROJECT_STATUS
}
