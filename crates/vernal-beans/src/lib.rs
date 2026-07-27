#![forbid(unsafe_code)]
#![doc = "Vernal 的 Tokio-first 类型驱动控制反转内核。"]

// ── Spring 风格模块（新迁移）──────────────────────────────────────────────

pub mod autowire;
pub mod aware;
pub mod bean_definition;
pub mod bean_definition_registry;
pub mod bean_definition_utils;
pub mod bean_expression_resolver;
pub mod bean_factory;
pub mod bean_factory_aware;
pub mod bean_factory_post_processor;
pub mod bean_name_aware;
pub mod bean_post_processor;
pub mod bean_scope;
pub mod configurable_bean_factory;
pub mod configurable_listable_bean_factory;
pub mod constructor_argument_values;
pub mod dependency_descriptor;
pub mod destruction_aware_bean_post_processor;
pub mod disposable_bean;
pub mod factory_bean;
pub mod hierarchical_bean_factory;
pub mod injection_point;
pub mod initializing_bean;
pub mod instantiation_aware_bean_post_processor;
pub mod listable_bean_factory;
pub mod mutable_property_values;
pub mod named_bean_holder;
pub mod object_provider;
pub mod property_value;
pub mod smart_instantiation_aware_bean_post_processor;
pub mod smart_initializing_singleton;
pub mod singleton_bean_registry;
pub mod type_converter;

// ── 原有 vernal-beans 模块 ─────────────────────────────────────────────

mod bean_desc_cache;
mod bean_descriptor;
mod bean_util;
mod build_plan;
mod component_contract;
mod component_definition;
mod component_key;
mod component_provider;
mod component_registry;
mod component_scope;
mod component_snapshot;
pub(crate) mod container;
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
mod trait_provider;
mod transient_tracker;

// ── Spring 风格 re-export ───────────────────────────────────────────────

pub use autowire::Autowire;
pub use bean_definition::BeanDefinition;
pub use bean_definition_registry::BeanDefinitionRegistry;
pub use bean_factory::{BeanFactory, FACTORY_BEAN_PREFIX};
pub use bean_factory_aware::BeanFactoryAware;
pub use bean_factory_post_processor::BeanFactoryPostProcessor;
pub use bean_name_aware::BeanNameAware;
pub use bean_post_processor::BeanPostProcessor;
pub use bean_scope::BeanScope;
pub use configurable_bean_factory::{ConfigurableBeanFactory, SCOPE_PROTOTYPE, SCOPE_SINGLETON};
pub use configurable_listable_bean_factory::ConfigurableListableBeanFactory;
pub use constructor_argument_values::{ConstructorArgumentValues, ValueHolder};
pub use dependency_descriptor::DependencyDescriptor;
pub use destruction_aware_bean_post_processor::DestructionAwareBeanPostProcessor;
pub use disposable_bean::DisposableBean;
pub use factory_bean::{FactoryBean, SmartFactoryBean};
pub use hierarchical_bean_factory::HierarchicalBeanFactory;
pub use injection_point::InjectionPoint;
pub use initializing_bean::InitializingBean;
pub use instantiation_aware_bean_post_processor::InstantiationAwareBeanPostProcessor;
pub use listable_bean_factory::ListableBeanFactory;
pub use mutable_property_values::MutablePropertyValues;
pub use named_bean_holder::NamedBeanHolder;
pub use object_provider::ObjectProvider;
pub use property_value::PropertyValue;
pub use smart_instantiation_aware_bean_post_processor::SmartInstantiationAwareBeanPostProcessor;
pub use smart_initializing_singleton::SmartInitializingSingleton;
pub use singleton_bean_registry::SingletonBeanRegistry;
pub use type_converter::TypeConverter;

// ── 原有 vernal-beans re-export ──────────────────────────────────────────

pub use bean_desc_cache::BeanDescCache;
pub use bean_descriptor::{BeanDescriptor, PropertyDescriptor};
pub use bean_util::{BeanError, BeanUtil};
pub use build_plan::BuildPlan;
pub use component_contract::Component;
pub use component_definition::ComponentDefinition;
pub use component_key::ComponentKey;
pub use component_provider::ComponentProvider;
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
pub use trait_provider::TraitProvider;
pub use transient_tracker::TransientTracker;

/// 返回当前 `IoC` 内核的成熟度状态。
#[must_use]
pub const fn project_status() -> &'static str {
    vernal_core::PROJECT_STATUS
}
