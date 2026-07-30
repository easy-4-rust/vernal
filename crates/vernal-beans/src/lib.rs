#![forbid(unsafe_code)]
#![doc = "Vernal 的 Tokio-first 类型驱动控制反转内核。"]

// ── Spring 风格模块 ───────────────────────────────────────────────

pub mod application_scope;
pub mod autowire;
pub mod autowire_capable_bean_factory;
pub mod aware;
pub mod bean_definition;
pub mod bean_definition_builder;
pub mod bean_definition_registry;
pub mod bean_definition_registry_post_processor;
pub mod bean_definition_utils;
pub mod bean_expression_resolver;
pub mod bean_factory;
pub mod bean_factory_aware;
pub mod bean_factory_utils;
pub mod bean_name_aware;
pub mod bean_post_processor;
pub mod bean_scope;
pub mod configurable_bean_factory;
pub mod configurable_listable_bean_factory;
pub mod configuration_class_post_processor;
pub mod constructor_argument_values;
pub mod conversion_service;
pub mod dependency_descriptor;
pub mod disposable_bean;
pub mod factory_bean;
pub mod field_metadata;
pub mod generic_bean_definition;
pub mod hierarchical_bean_factory;
pub mod initializing_bean;
pub mod injection_point;
pub mod listable_bean_factory;
pub mod mutable_property_values;
pub mod named_bean_holder;
pub mod object_provider;
pub mod property_editor;
pub mod property_value;
pub mod request_scope;
pub mod root_bean_definition;
pub mod session_scope;
pub mod singleton_bean_registry;
pub mod smart_initializing_singleton;
pub mod standard_bean_expression_resolver;
pub mod type_converter;
pub mod type_converter_delegate;
pub mod boolean_editor;
pub mod byte_array_editor;
pub mod byte_array_property_editor;
pub mod char_array_editor;
pub mod char_array_property_editor;
pub mod char_property_editor;
pub mod charset_editor;
pub mod charset_property_editor;
pub mod class_array_editor;
pub mod class_editor;
pub mod currency_editor;
pub mod file_array_editor;
pub mod file_editor;
pub mod input_source_editor;
pub mod input_stream_editor;
pub mod locale_editor;
pub mod number_editor;
pub mod path_editor;
pub mod path_property_editor;
pub mod pattern_editor;
pub mod properties_editor;
pub mod reader_editor;
pub mod resource_bundle_editor;
pub mod string_array_editor;
pub mod string_trimmer_editor;
pub mod timezone_editor;
pub mod uri_editor;
pub mod uuid_editor;
pub mod zone_id_editor;
pub mod bean_definition_override_exception;
pub mod bean_definition_parsing_exception;
pub mod bean_definition_validation_exception;
pub mod bean_is_abstract_exception;
pub mod bean_is_not_a_factory_exception;
pub mod factory_bean_not_initialized_exception;
pub mod type_mismatch_exception;
pub mod conversion_not_supported_exception;
pub mod scope_not_active_exception;
pub mod property_batch_update_exception;
pub mod xml_bean_definition_store_exception;
pub mod aot_processing_exception;
pub mod aot_bean_processing_exception;
pub mod aot_exception;

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
mod scope_context;
mod scope_error;
mod scope_key;
mod scope_operation_guard;
mod scope_runtime_state;
mod trait_binding;
mod trait_binding_snapshot;
mod trait_key;
mod trait_provider;
mod transient_tracker;

// ── Spring 风格 re-export ───────────────────────────────────────────────

pub use application_scope::ApplicationScope;
pub use autowire::Autowire;
pub use autowire_capable_bean_factory::AutowireCapableBeanFactory;
pub use bean_definition::BeanDefinition;
pub use bean_definition_builder::BeanDefinitionBuilder;
pub use bean_definition_registry::BeanDefinitionRegistry;
pub use bean_factory::BeanFactory;
pub use bean_factory_aware::BeanFactoryAware;
pub use bean_name_aware::BeanNameAware;
pub use bean_post_processor::BeanPostProcessor;
pub use bean_scope::BeanScope;
pub use configurable_bean_factory::ConfigurableBeanFactory;
pub use configurable_listable_bean_factory::ConfigurableListableBeanFactory;
pub use constructor_argument_values::ConstructorArgumentValues;
pub use conversion_service::ConversionService;
pub use dependency_descriptor::DependencyDescriptor;
pub use disposable_bean::DisposableBean;
pub use factory_bean::FactoryBean;
pub use generic_bean_definition::GenericBeanDefinition;
pub use hierarchical_bean_factory::HierarchicalBeanFactory;
pub use initializing_bean::InitializingBean;
pub use injection_point::InjectionPoint;
pub use listable_bean_factory::ListableBeanFactory;
pub use mutable_property_values::MutablePropertyValues;
pub use named_bean_holder::NamedBeanHolder;
pub use object_provider::ObjectProvider;
pub use property_editor::PropertyEditor;
pub use property_value::PropertyValue;
pub use request_scope::RequestScope;
pub use root_bean_definition::RootBeanDefinition;
pub use session_scope::SessionScope;
pub use singleton_bean_registry::SingletonBeanRegistry;
pub use smart_initializing_singleton::SmartInitializingSingleton;
pub use type_converter::TypeConverter;

// ── 原有 vernal-beans re-export ──────────────────────────────────────────

pub use bean_desc_cache::BeanDescCache;
pub use bean_descriptor::BeanDescriptor;
pub use bean_util::BeanError;
pub use bean_util::BeanUtil;
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
pub use scope_key::ScopeKey;
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

// ── 新增核心模块 ─────────────────────────────────────────────────────
pub mod bean_factory_post_processor;
pub mod scope_state;
pub mod scope_close_failure;
pub mod scope_future;
pub use scope_state::ScopeState;
