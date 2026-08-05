#![forbid(unsafe_code)]
#![doc = "Vernal 的 Tokio-first 类型驱动控制反转内核。"]

// ── Spring 风格模块 ───────────────────────────────────────────────

pub mod method_invocation_exception;
pub mod property_accessor_utils;
pub mod property_matches;
pub mod simple_type_converter;

pub mod application_scope;
pub mod bean_class_loader_aware;
pub mod bean_definition_utils;
pub mod bean_info_factory;
pub mod bean_metadata_attribute;
pub mod bean_metadata_attribute_accessor;
pub mod bean_metadata_element;
pub mod bean_scope;
pub mod bean_utils;
pub mod bean_utils_runtime_hints;
pub mod beans_exception;
pub mod boolean_editor;
pub mod byte_array_editor;
pub mod cached_introspection_results;
pub mod char_array_editor;
pub mod char_property_editor;
pub mod charset_property_editor;
pub mod configuration_class_post_processor;
pub mod conversion_not_supported_exception;
pub mod conversion_service;
pub mod destructible_bean_adapter;
pub mod direct_field_accessor;
pub mod extended_bean_info;
pub mod extended_bean_info_factory;
pub mod field_metadata;
pub mod file_array_editor;
pub mod mergeable;
pub mod mutable_property_values;
pub mod number_editor;
pub mod path_property_editor;
pub mod property_accessor_factory;
pub mod property_batch_update_exception;
pub mod property_descriptor_utils;
pub mod property_editor;
pub mod property_editor_registrar;
pub mod property_editor_registry;
pub mod property_editor_registry_support;
pub mod property_value;
pub mod property_values;
pub mod property_values_editor;
pub mod request_scope;
pub mod session_scope;
pub mod simple_bean_info_factory;
pub mod standard_bean_expression_resolver;
pub mod standard_bean_info_factory;
pub mod string_array_editor;
pub mod timezone_editor;
pub mod type_converter;
pub mod type_converter_delegate;
pub mod type_converter_support;
pub mod type_mismatch_exception;

// ── 属性访问机制模块 ───────────────────────────────────────────────────

pub mod abstract_nestable_property_accessor;
pub mod abstract_property_accessor;
pub mod bean_wrapper;
pub mod bean_wrapper_impl;
pub mod configurable_property_accessor;
pub mod property_accessor;

// ── 原有 vernal-beans 模块 ─────────────────────────────────────────────

mod bean_desc_cache;
mod bean_descriptor;
mod bean_util;
mod build_plan;
mod component_contract;
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

// ── 子包模块 ─────────────────────────────────────────────────────────

/// 对应 Spring beans.factory 包。
pub mod factory;

/// 对应 Spring beans.propertyeditors 包。
pub mod propertyeditors;

/// 对应 Spring beans.support 包。
pub mod support;

// ── 新增核心模块 ─────────────────────────────────────────────────────
pub mod bean_instantiation_exception;
pub mod destruction_aware_bean_post_processor;
pub mod fatal_bean_exception;
pub mod instantiation_aware_bean_post_processor;
pub mod invalid_property_exception;
pub mod not_readable_property_exception;
pub mod not_writable_property_exception;
pub mod null_value_in_nested_path_exception;
pub mod property_access_exception;
pub mod scope_close_failure;
pub mod scope_future;
pub mod scope_state;

// ── 新增 Bean 生命周期处理模块 ───────────────────────────────────────
pub mod bean_factory_aware_processor;
pub mod bean_name_aware_processor;
pub mod class_loader_aware_processor;
pub mod destruction_bean_post_processor;
pub mod initialization_bean_post_processor;

// ── 新增类型转换和编辑器模块 ─────────────────────────────────────────
pub mod bean_wrapper_info;
pub mod conversion_service_factory;
pub mod property_editor_cache;
pub mod type_editor_registry;

// ── 新增异常类型 ────────────────────────────────────────────────────
pub mod bean_factory_access_exception;

// ── Bean 描述符与泛型类型支持 ─────────────────────────────────────────
pub mod bean_creation_error;
pub mod bean_descriptor_utils;
pub mod generic_type_aware_property_descriptor;

// ── Spring 风格 re-export ───────────────────────────────────────────────

pub use application_scope::ApplicationScope;
pub use bean_scope::BeanScope;
pub use configuration_class_post_processor::ConfigurationClassPostProcessor;
pub use conversion_service::ConversionService;
pub use destructible_bean_adapter::DisposableBeanAdapter;
pub use factory::annotation::autowire::Autowire;
pub use mutable_property_values::MutablePropertyValues;
pub use property_editor::PropertyEditor;
pub use property_editor_registry::PropertyEditorRegistry;
pub use property_editor_registry_support::PropertyEditorRegistrySupport;
pub use property_value::PropertyValue;
pub use request_scope::RequestScope;
pub use session_scope::SessionScope;
pub use type_converter::TypeConverter;

// ── factory 子包 re-export ─────────────────────────────────────────────

// factory 根模块
pub use factory::aware::Aware;
pub use factory::bean_factory::BeanFactory;
pub use factory::bean_factory_aware::BeanFactoryAware;
pub use factory::bean_name_aware::BeanNameAware;
pub use factory::disposable_bean::DisposableBean;
pub use factory::factory_bean::FactoryBean;
pub use factory::hierarchical_bean_factory::HierarchicalBeanFactory;
pub use factory::initializing_bean::InitializingBean;
pub use factory::injection_point::InjectionPoint;
pub use factory::listable_bean_factory::ListableBeanFactory;
pub use factory::object_provider::ObjectProvider;
pub use factory::smart_initializing_singleton::SmartInitializingSingleton;

// factory 异常类型
pub use factory::bean_creation_exception::BeanCreationException;
pub use factory::bean_creation_not_allowed_exception::BeanCreationNotAllowedException;
pub use factory::bean_currently_in_creation_exception::BeanCurrentlyInCreationException;
pub use factory::bean_definition_store_exception::BeanDefinitionStoreException;
pub use factory::bean_expression_exception::BeanExpressionException;
pub use factory::bean_initialization_exception::BeanInitializationException;
pub use factory::bean_not_of_required_type_exception::BeanNotOfRequiredTypeException;
pub use factory::bean_registrar::BeanRegistrar;
pub use factory::bean_registry::BeanRegistry;
pub use factory::cannot_load_bean_class_exception::CannotLoadBeanClassException;
pub use factory::named_bean::NamedBean;
pub use factory::no_such_bean_definition_exception::NoSuchBeanDefinitionException;
pub use factory::no_unique_bean_definition_exception::NoUniqueBeanDefinitionException;
pub use factory::object_factory::ObjectFactory;
pub use factory::smart_factory_bean::SmartFactoryBean;
pub use factory::unsatisfied_dependency_exception::UnsatisfiedDependencyException;

// factory/config 模块
pub use factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
pub use factory::config::bean_definition::BeanDefinition;
pub use factory::config::bean_definition_holder::BeanDefinitionHolder;
pub use factory::config::bean_expression_resolver::BeanExpressionResolver;
pub use factory::config::bean_factory_post_processor::BeanFactoryPostProcessor;
pub use factory::config::bean_post_processor::BeanPostProcessor;
pub use factory::config::configurable_bean_factory::ConfigurableBeanFactory;
pub use factory::config::configurable_listable_bean_factory::ConfigurableListableBeanFactory;
pub use factory::config::constructor_argument_values::ConstructorArgumentValues;
pub use factory::config::dependency_descriptor::DependencyDescriptor;
pub use factory::config::named_bean_holder::NamedBeanHolder;
pub use factory::config::singleton_bean_registry::SingletonBeanRegistry;

// factory/support 模块
pub use factory::support::bean_definition_builder::BeanDefinitionBuilder;
pub use factory::support::bean_definition_registry::BeanDefinitionRegistry;
pub use factory::support::constructor_resolver::ConstructorResolver;
pub use factory::support::default_singleton_bean_registry::DefaultSingletonBeanRegistry;
pub use factory::support::factory_bean_registry_support::FactoryBeanRegistrySupport;
pub use factory::support::generic_bean_definition::GenericBeanDefinition;
pub use factory::support::instantiation_strategy::InstantiationStrategy;
pub use factory::support::root_bean_definition::RootBeanDefinition;
pub use factory::support::simple_instantiation_strategy::SimpleInstantiationStrategy;

// ── 原有 vernal-beans re-export ──────────────────────────────────────────

pub use bean_class_loader_aware::BeanClassLoaderAware;
pub use bean_desc_cache::BeanDescCache;
pub use bean_descriptor::BeanDescriptor;
pub use bean_descriptor::PropertyDescriptor;
pub use bean_info_factory::BeanInfoFactory;
pub use bean_metadata_attribute::BeanMetadataAttribute;
pub use bean_metadata_attribute_accessor::BeanMetadataAttributeAccessor;
pub use bean_metadata_element::BeanMetadataElement;
pub use bean_util::BeanError;
pub use bean_util::BeanUtil;
pub use bean_utils::BeanUtils;
pub use beans_exception::BeansException;
pub use build_plan::BuildPlan;
pub use cached_introspection_results::CachedIntrospectionResults;
pub use component_contract::Component;
pub use component_key::ComponentKey;
pub use component_provider::ComponentProvider;
pub use component_registry::Registry;
pub use component_scope::Scope;
pub use component_snapshot::ComponentSnapshot;
pub use container::Container;
pub use definition_error::DefinitionError;
pub use dependency::Dependency;
pub use direct_field_accessor::DirectFieldAccessor;
pub use extended_bean_info::ExtendedBeanInfo;
pub use extended_bean_info_factory::ExtendedBeanInfoFactory;
pub use factory::annotation::qualifier::Qualifier;
pub use factory::parsing::component_definition::ComponentDefinition;
pub use graph_error::GraphError;
pub use mergeable::Mergeable;
pub use property_accessor_factory::PropertyAccessorFactory;
pub use property_editor_registrar::PropertyEditorRegistrar;
pub use property_values::EmptyPropertyValues;
pub use property_values::MutablePropertyValuesImpl;
pub use property_values::PropertyValues;
pub use registry_builder::RegistryBuilder;
pub use registry_snapshot::RegistrySnapshot;
pub use registry_summary::RegistrySummary;
pub use resolve_error::ResolveError;
pub use resolver::Resolver;
pub use scope_context::ScopeContext;
pub use scope_error::ScopeError;
pub use scope_key::ScopeKey;
pub use scope_state::ScopeState;
pub use simple_bean_info_factory::SimpleBeanInfoFactory;
pub use standard_bean_info_factory::StandardBeanInfoFactory;
pub use trait_binding::TraitBinding;
pub use trait_binding_snapshot::TraitBindingSnapshot;
pub use trait_key::TraitKey;
pub use trait_provider::TraitProvider;
pub use transient_tracker::TransientTracker;

// ── 新增类型 re-export ────────────────────────────────────────────────
pub use bean_factory_access_exception::BeanFactoryAccessException;
pub use bean_factory_aware_processor::BeanFactoryAwareProcessor;
pub use bean_name_aware_processor::BeanNameAwareProcessor;
pub use bean_wrapper_info::BeanWrapperInfo;
pub use class_loader_aware_processor::ClassLoaderAwareProcessor;
pub use conversion_service_factory::ConversionServiceFactory;
pub use destruction_bean_post_processor::DestructionBeanPostProcessor;
pub use initialization_bean_post_processor::InitializationBeanPostProcessor;
pub use property_editor_cache::PropertyEditorCache;
pub use type_converter_support::TypeConverterSupport;
pub use type_editor_registry::TypeEditorRegistry;

// ── 子包模块 re-export（供测试和外部使用） ─────────────────────────────

// factory 根模块
pub use factory::bean_factory_utils::BeanFactoryUtils;
pub use factory::bean_is_abstract_exception::BeanIsAbstractException;
pub use factory::bean_is_not_a_factory_exception::BeanIsNotAFactoryException;
pub use factory::factory_bean_not_initialized_exception::FactoryBeanNotInitializedException;

// factory/aot 模块
pub use factory::aot::aot_bean_processing_exception::AotBeanProcessingException;
pub use factory::aot::aot_exception::AotException;
pub use factory::aot::aot_processing_exception::AotProcessingException;

// factory/config 模块
pub use factory::config::custom_editor_configurer::CustomEditorConfigurer;

// factory/parsing 模块
pub use factory::parsing::bean_definition_parsing_exception::BeanDefinitionParsingException;

// factory/support 模块
pub use factory::support::bean_definition_override_exception::BeanDefinitionOverrideException;
pub use factory::support::bean_definition_validation_exception::BeanDefinitionValidationException;
pub use factory::support::scope_not_active_exception::ScopeNotActiveException;
pub use factory::support::simple_bean_definition_registry::SimpleBeanDefinitionRegistry;
pub use factory::support::static_listable_bean_factory::StaticListableBeanFactory;

// factory/xml 模块
pub use factory::xml::xml_bean_definition_store_exception::XmlBeanDefinitionStoreException;

// propertyeditors 模块
pub use propertyeditors::string_trimmer_editor::StringTrimmerEditor;
pub use propertyeditors::uri_editor::URIEditor;
pub use propertyeditors::uuid_editor::UUIDEditor;
pub use propertyeditors::zone_id_editor::ZoneIdEditor;

/// 返回当前 `IoC` 内核的成熟度状态。
#[must_use]
pub const fn project_status() -> &'static str {
    vernal_core::PROJECT_STATUS
}
