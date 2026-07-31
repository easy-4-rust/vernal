//! factory::config — 对应 Spring beans.factory.config 包。
//!
//! 包含 Bean 工厂配置接口和实现。

/// Spring AutowireCapableBeanFactory 接口。
pub mod autowire_capable_bean_factory;

/// Spring BeanDefinition 接口。
pub mod bean_definition;

/// Spring BeanDefinitionHolder 类。
pub mod bean_definition_holder;

/// Spring BeanExpressionResolver 接口。
pub mod bean_expression_resolver;

/// Spring BeanFactoryPostProcessor 接口。
pub mod bean_factory_post_processor;

/// Spring BeanPostProcessor 接口。
pub mod bean_post_processor;

/// Spring ConfigurableBeanFactory 接口。
pub mod configurable_bean_factory;

/// Spring ConfigurableListableBeanFactory 接口。
pub mod configurable_listable_bean_factory;

/// Spring ConstructorArgumentValues 类。
pub mod constructor_argument_values;

/// Spring CustomEditorConfigurer 类。
pub mod custom_editor_configurer;

/// Spring DependencyDescriptor 类。
pub mod dependency_descriptor;

/// Spring NamedBeanHolder 类。
pub mod named_bean_holder;

/// Spring SingletonBeanRegistry 接口。
pub mod singleton_bean_registry;

/// Spring BeanRegistrationAotProcessor 接口。
pub mod bean_registration_aot_processor;

/// Spring AbstractFactoryBean 类。
pub mod abstract_factory_bean;

/// Spring AutowiredPropertyMarker 类。
pub mod autowired_property_marker;

/// Spring BeanDefinitionCustomizer 接口。
pub mod bean_definition_customizer;

/// Spring BeanDefinitionVisitor 类。
pub mod bean_definition_visitor;

/// Spring BeanExpressionContext 类。
pub mod bean_expression_context;

/// Spring BeanReference 类。
pub mod bean_reference;

/// Spring CustomScopeConfigurer 类。
pub mod custom_scope_configurer;

/// Spring DeprecatedBeanWarner 类。
pub mod deprecated_bean_warner;

/// Spring DestructionAwareBeanPostProcessor 接口。
pub mod destruction_aware_bean_post_processor;

/// Spring EmbeddedValueResolver 类。
pub mod embedded_value_resolver;

/// Spring Scope 接口。
pub mod scope;

/// Spring RuntimeBeanReference 类。
pub mod runtime_bean_reference;

/// Spring RuntimeBeanNameReference 类。
pub mod runtime_bean_name_reference;

/// Spring TypedStringValue 类。
pub mod typed_string_value;

/// Spring ListFactoryBean 类。
pub mod list_factory_bean;

/// Spring SetFactoryBean 类。
pub mod set_factory_bean;

/// Spring MapFactoryBean 类。
pub mod map_factory_bean;

/// Spring PropertiesFactoryBean 类。
pub mod properties_factory_bean;

/// Spring FieldRetrievingFactoryBean 类。
pub mod field_retrieving_factory_bean;

/// Spring MethodInvokingBean 类。
pub mod method_invoking_bean;

/// Spring MethodInvokingFactoryBean 类。
pub mod method_invoking_factory_bean;

/// Spring PropertyPathFactoryBean 类。
pub mod property_path_factory_bean;

/// Spring PlaceholderConfigurerSupport 类。
pub mod placeholder_configurer_support;

/// Spring PropertyResourceConfigurer 类。
pub mod property_resource_configurer;

/// Spring PropertyPlaceholderConfigurer 类。
pub mod property_placeholder_configurer;

/// Spring PropertyOverrideConfigurer 类。
pub mod property_override_configurer;

/// Spring PreferencesPlaceholderConfigurer 类。
pub mod preferences_placeholder_configurer;

/// Spring ServiceLocatorFactoryBean 类。
pub mod service_locator_factory_bean;

/// Spring ProviderCreatingFactoryBean 类。
pub mod provider_creating_factory_bean;

/// Spring ObjectFactoryCreatingFactoryBean 类。
pub mod object_factory_creating_factory_bean;

/// Spring YamlProcessor 类。
pub mod yaml_processor;

/// Spring YamlPropertiesFactoryBean 类。
pub mod yaml_properties_factory_bean;

/// Spring YamlMapFactoryBean 类。
pub mod yaml_map_factory_bean;

/// Spring InstantiationAwareBeanPostProcessor 接口。
pub mod instantiation_aware_bean_post_processor;

/// Spring SmartInstantiationAwareBeanPostProcessor 接口。
pub mod smart_instantiation_aware_bean_post_processor;
