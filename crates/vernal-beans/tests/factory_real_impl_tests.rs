/// Tests for factory/support, factory/annotation, factory/aot implementations.
use std::sync::Arc;

use std::any::TypeId;

// ═══════════════════════════════════════════════════════════════════
// factory/support/ tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn autowire_utils_static() {
    use vernal_beans::factory::support::autowire_utils::AutowireUtils;
    assert!(AutowireUtils::is_autowire_type("set"));
    assert!(AutowireUtils::is_autowire_type("get"));
    assert!(AutowireUtils::is_autowire_type("is"));
    assert!(!AutowireUtils::is_autowire_type("foo"));
    assert_eq!(AutowireUtils::resolve_autowire_value("setName"), "name");
}

#[test]
fn default_singleton_registry_l1() {
    use std::sync::Arc;
    use vernal_beans::factory::support::default_singleton_bean_registry::DefaultSingletonBeanRegistry;
    let r = DefaultSingletonBeanRegistry::new();
    assert_eq!(r.getSingletonCount(), 0);
    r.registerSingleton("b1".to_string(), Arc::new(42i32));
    assert_eq!(r.getSingletonCount(), 1);
    assert!(r.containsSingleton("b1"));
}

#[test]
fn default_singleton_registry_l3() {
    use std::sync::Arc;
    use vernal_beans::factory::support::default_singleton_bean_registry::DefaultSingletonBeanRegistry;
    let r = DefaultSingletonBeanRegistry::new();
    r.addSingletonFactory("b1".to_string(), Arc::new(|| Arc::new(99i32) as Arc<dyn std::any::Any + Send + Sync>));
    let early = r.getEarlyBeanReference("b1");
    assert!(early.is_some());
}

#[test]
fn default_singleton_registry_lifecycle() {
    use vernal_beans::factory::support::default_singleton_bean_registry::DefaultSingletonBeanRegistry;
    let r = DefaultSingletonBeanRegistry::new();
    assert!(!r.isCurrentlyInCreation("b1"));
    r.markAsInCreation("b1");
    assert!(r.isCurrentlyInCreation("b1"));
}

#[test]
fn bean_definition_defaults_test() {
    use vernal_beans::factory::support::bean_definition_defaults::BeanDefinitionDefaults;
    let mut d = BeanDefinitionDefaults::new();
    assert!(!d.lazyInit);
    d.set_lazy_init(true);
    assert!(d.lazyInit);
}

#[test]
fn autowire_candidate_qualifier_test() {
    use vernal_beans::factory::support::autowire_candidate_qualifier::AutowireCandidateQualifier;
    let q = AutowireCandidateQualifier::new("javax.inject.Qualifier".to_string());
    assert_eq!(q.qualifier_type(), "javax.inject.Qualifier");
    q.set_attribute("value".to_string(), "primary".to_string());
    assert_eq!(q.get_attribute("value"), Some("primary".to_string()));
}

#[test]
fn bean_definition_reader_utils_test() {
    use vernal_beans::factory::support::bean_definition_reader_utils::BeanDefinitionReaderUtils;
    let u = BeanDefinitionReaderUtils::new();
    assert_eq!(u.generate_bean_name("myBean"), "myBean#1");
    assert_eq!(u.generate_bean_name("myBean"), "myBean#2");
}

#[test]
fn bean_definition_resource_test() {
    use vernal_beans::factory::support::bean_definition_resource::BeanDefinitionResource;
    let r = BeanDefinitionResource::new("test.xml".to_string(), "<beans/>".to_string());
    assert_eq!(r.description(), "test.xml");
    assert!(!r.is_empty());
}

#[test]
fn bean_definition_value_resolver_test() {
    use vernal_beans::factory::support::bean_definition_value_resolver::BeanDefinitionValueResolver;
    let r = BeanDefinitionValueResolver::new();
    r.resolve("name".to_string(), "Alice".to_string());
    assert_eq!(r.get("name"), Some("Alice".to_string()));
}

#[test]
fn default_bean_name_generator_trait() {
    use vernal_beans::factory::support::bean_name_generator::BeanNameGenerator;
    use vernal_beans::factory::support::default_bean_name_generator::DefaultBeanNameGenerator;
    let g = DefaultBeanNameGenerator::new();
    let n1 = g.generate_bean_name(TypeId::of::<String>());
    let n2 = g.generate_bean_name(TypeId::of::<String>());
    assert_ne!(n1, n2);
}

#[test]
fn bean_registry_adapter_test() {
    use vernal_beans::factory::support::bean_registry_adapter::BeanRegistryAdapter;
    let a = BeanRegistryAdapter::new();
    assert_eq!(a.count(), 0);
    a.register("myBean".to_string(), TypeId::of::<String>());
    assert_eq!(a.count(), 1);
    a.unregister("myBean");
    assert_eq!(a.count(), 0);
}

#[test]
fn abstract_autowire_capable_bean_factory_test() {
    use vernal_beans::factory::support::abstract_autowire_capable_bean_factory::AbstractAutowireCapableBeanFactory;
    let f = AbstractAutowireCapableBeanFactory::new();
    f.ignore_dependency_type(TypeId::of::<String>());
    assert!(f.is_dependency_ignored(TypeId::of::<String>()));
    assert_eq!(f.ignored_count(), 1);
}

#[test]
fn abstract_bean_definition_reader_test() {
    use vernal_beans::factory::support::abstract_bean_definition_reader::AbstractBeanDefinitionReader;
    let mut r = AbstractBeanDefinitionReader::new();
    r.increment_bean_class_count();
    r.increment_bean_class_count();
    assert_eq!(r.get_bean_class_count(), 2);
}

#[test]
fn generic_type_aware_resolver_test() {
    use vernal_beans::factory::support::autowire_candidate_resolver::AutowireCandidateResolver;
    use vernal_beans::factory::support::generic_type_aware_autowire_candidate_resolver::GenericTypeAwareAutowireCandidateResolver;
    let r = GenericTypeAwareAutowireCandidateResolver::new();
    assert!(r.is_autowire_candidate(TypeId::of::<String>(), "bean"));
    r.exclude_type(TypeId::of::<String>());
    assert!(!r.is_autowire_candidate(TypeId::of::<String>(), "bean"));
}

#[test]
fn disposable_bean_adapter_test() {
    use std::sync::Arc;
    use vernal_beans::factory::support::disposable_bean_adapter::DisposableBeanAdapter;
    let adapter = DisposableBeanAdapter::new("myBean".to_string(), Arc::new(42i32))
        .with_destroy_method("destroy".to_string());
    assert_eq!(adapter.bean_name(), "myBean");
    assert_eq!(adapter.destroy_method_name(), Some("destroy"));
}

#[test]
fn managed_collections_test() {
    use std::sync::Arc;
    use vernal_beans::factory::support::managed_list::ManagedList;
    use vernal_beans::factory::support::managed_map::ManagedMap;
    use vernal_beans::factory::support::managed_set::ManagedSet;
    use vernal_beans::factory::support::managed_array::ManagedArray;
    use vernal_beans::factory::support::managed_properties::ManagedProperties;
    
    let list = ManagedList::new();
    list.add(Arc::new(1i32) as Arc<dyn std::any::Any + Send + Sync>);
    list.add(Arc::new(2i32) as Arc<dyn std::any::Any + Send + Sync>);
    assert_eq!(list.len(), 2);
    
    let map = ManagedMap::new();
    map.put("k1".to_string(), Arc::new(1) as Arc<dyn std::any::Any + Send + Sync>);
    assert!(map.contains_key("k1"));
    
    let set = ManagedSet::new();
    set.add("a");
    set.add("b");
    set.add("a");
    assert_eq!(set.len(), 2);
    
    let arr = ManagedArray::new();
    arr.add(Arc::new(1) as Arc<dyn std::any::Any + Send + Sync>);
    assert_eq!(arr.len(), 1);
    
    let p = ManagedProperties::new();
    p.set("k1".to_string(), "v1".to_string());
    assert_eq!(p.get("k1"), Some("v1".to_string()));
}

// ═══════════════════════════════════════════════════════════════════
// factory/annotation/ tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn annotated_bean_definition_test() {
    use vernal_beans::factory::annotation::annotated_bean_definition::{
        AnnotatedBeanDefinition, GenericAnnotatedBeanDefinition,
    };
    let def = GenericAnnotatedBeanDefinition::new("com.example.Service".to_string())
        .with_factory_method("create".to_string())
        .with_annotation_count(3);
    assert_eq!(def.bean_class_name(), "com.example.Service");
    assert!(def.is_factory_method("create"));
}

#[test]
fn autowired_annotation_test() {
    use vernal_beans::factory::annotation::autowired::Autowired;
    let mut a = Autowired::new();
    assert!(a.required());
    a.set_required(false);
    a.set_primary(true);
    assert!(a.is_primary());
    a.add_qualifier_type(TypeId::of::<String>());
    assert_eq!(a.qualifier_count(), 1);
}

#[test]
fn annotation_markers_test() {
    use vernal_beans::factory::annotation::value::Value;
    use vernal_beans::factory::annotation::configurable::Configurable;
    use vernal_beans::factory::annotation::lookup::Lookup;
    
    let v = Value::new("hello".to_string());
    assert_eq!(v.value(), "hello");
    
    let mut c = Configurable::new();
    assert!(c.enabled());
    c.set_enabled(false);
    assert!(!c.enabled());
    
    let l = Lookup::new("myBean".to_string());
    assert_eq!(l.value(), "myBean");
}

#[test]
fn custom_autowire_configurer_test() {
    use vernal_beans::factory::annotation::custom_autowire_configurer::CustomAutowireConfigurer;
    let c = CustomAutowireConfigurer::new();
    c.set_required(false);
    c.add_custom_qualifier(TypeId::of::<String>());
    assert_eq!(c.custom_qualifier_count(), 1);
}

#[test]
fn parameter_resolution_delegate_test() {
    use vernal_beans::factory::annotation::parameter_resolution_delegate::ParameterResolutionDelegate;
    let mut d = ParameterResolutionDelegate::new();
    d.register_dependency("name".to_string(), TypeId::of::<String>());
    d.increment_resolved();
    assert_eq!(d.resolved_count(), 1);
}

#[test]
fn jakarta_annotations_runtime_hints_test() {
    use vernal_beans::factory::annotation::jakarta_annotations_runtime_hints::JakartaAnnotationsRuntimeHints;
    let h = JakartaAnnotationsRuntimeHints::new();
    h.register();
    assert_eq!(h.registered_count(), 1);
}

#[test]
fn bean_factory_annotation_utils_test() {
    use vernal_beans::factory::annotation::bean_factory_annotation_utils::BeanFactoryAnnotationUtils;
    let u = BeanFactoryAnnotationUtils::new();
    u.mark_processed("Autowired".to_string());
    assert!(u.is_processed("Autowired"));
}

#[test]
fn annotation_bean_wiring_info_resolver_test() {
    use vernal_beans::factory::annotation::annotation_bean_wiring_info_resolver::AnnotationBeanWiringInfoResolver;
    let r = AnnotationBeanWiringInfoResolver::new();
    r.register_wiring_info(TypeId::of::<String>(), vec!["name".to_string()]);
    let info = r.get_wiring_info(TypeId::of::<String>()).unwrap();
    assert_eq!(info.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════
// factory/aot/ tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn aot_basic_test() {
    use vernal_beans::factory::aot::aot_services::AotServices;
    use vernal_beans::factory::aot::autowired_arguments_code_generator::AutowiredArgumentsCodeGenerator;
    use vernal_beans::factory::aot::code_warnings::CodeWarnings;
    
    let s = AotServices::new();
    s.register("k".to_string(), "v".to_string());
    assert_eq!(s.count(), 1);
    
    let g = AutowiredArgumentsCodeGenerator::new();
    let r = g.process("test".to_string());
    assert!(!r.is_empty());
    
    let w = CodeWarnings::new();
    w.register("w1".to_string(), "msg1".to_string());
    assert_eq!(w.cache_size(), 1);
}

#[test]
fn bean_registration_aot_test() {
    use vernal_beans::factory::aot::bean_registration_aot_contribution::BeanRegistrationAotContribution;
    use vernal_beans::factory::aot::bean_registration_code::BeanRegistrationCode;
    use vernal_beans::factory::aot::bean_registration_code_fragments::BeanRegistrationCodeFragments;
    use vernal_beans::factory::aot::bean_registration_code_generator::BeanRegistrationCodeGenerator;
    
    let c = BeanRegistrationAotContribution::new();
    c.register("bean1".to_string(), "method1".to_string());
    assert_eq!(c.get("bean1"), Some("method1".to_string()));
    
    let code = BeanRegistrationCode::new();
    code.register("code".to_string(), "body".to_string());
    assert_eq!(code.get("code"), Some("body".to_string()));
    
    let f = BeanRegistrationCodeFragments::new();
    f.register("frag".to_string(), "data".to_string());
    assert_eq!(f.get("frag"), Some("data".to_string()));
    
    let g = BeanRegistrationCodeGenerator::new();
    g.register("gen".to_string(), "code".to_string());
    assert_eq!(g.count(), 1);
}

#[test]
fn bean_aot_misc_test() {
    use vernal_beans::factory::aot::bean_instance_supplier::BeanInstanceSupplier;
    use vernal_beans::factory::aot::bean_registrations_aot_contribution::BeanRegistrationsAotContribution;
    use vernal_beans::factory::aot::bean_registrations_aot_processor::BeanRegistrationsAotProcessor;
    use vernal_beans::factory::aot::bean_registrations_code::BeanRegistrationsCode;
    use vernal_beans::factory::aot::bean_definition_method_generator::BeanDefinitionMethodGenerator;
    use vernal_beans::factory::aot::bean_definition_method_generator_factory::BeanDefinitionMethodGeneratorFactory;
    use vernal_beans::factory::aot::autowired_field_value_resolver::AutowiredFieldValueResolver;
    use vernal_beans::factory::aot::autowired_method_arguments_resolver::AutowiredMethodArgumentsResolver;
    use vernal_beans::factory::aot::autowired_element_resolver::AutowiredElementResolver;
    use vernal_beans::factory::aot::bean_definition_properties_code_generator::BeanDefinitionPropertiesCodeGenerator;
    use vernal_beans::factory::aot::bean_factory_initialization_aot_contribution::BeanFactoryInitializationAotContribution;
    use vernal_beans::factory::aot::bean_factory_initialization_code::BeanFactoryInitializationCode;
    use vernal_beans::factory::aot::bean_registration_exclude_filter::BeanRegistrationExcludeFilter;
    use vernal_beans::factory::aot::default_bean_registration_code_fragments::DefaultBeanRegistrationCodeFragments;
    use vernal_beans::factory::aot::bean_registration_code_fragments_decorator::BeanRegistrationCodeFragmentsDecorator;
    use vernal_beans::factory::aot::bean_definition_property_value_code_generator_delegates::BeanDefinitionPropertyValueCodeGeneratorDelegates;
    use vernal_beans::factory::aot::instance_supplier_code_generator::InstanceSupplierCodeGenerator;
    
    let s = BeanInstanceSupplier::new();
    s.register("k".to_string(), "v".to_string());
    assert_eq!(s.count(), 1);
    
    let c = BeanRegistrationsAotContribution::new();
    c.register("c".to_string(), "v".to_string());
    assert_eq!(c.count(), 1);
    
    let p = BeanRegistrationsAotProcessor::new();
    p.register("p".to_string(), "v".to_string());
    assert_eq!(p.count(), 1);
    
    let r = BeanRegistrationsCode::new();
    r.register("r".to_string(), "v".to_string());
    assert_eq!(r.count(), 1);
    
    let g = BeanDefinitionMethodGenerator::new();
    g.register("g".to_string(), "v".to_string());
    assert_eq!(g.cache_size(), 1);
    
    let f = BeanDefinitionMethodGeneratorFactory::new();
    f.register("f".to_string(), "v".to_string());
    assert!(f.contains("f"));
    
    let fr = AutowiredFieldValueResolver::new();
    fr.register("f".to_string(), "v".to_string());
    assert_eq!(fr.count(), 1);
    
    let m = AutowiredMethodArgumentsResolver::new();
    m.register("m".to_string(), "v".to_string());
    assert_eq!(m.count(), 1);
    
    let e = AutowiredElementResolver::new();
    e.register("e".to_string(), "v".to_string());
    assert!(e.contains("e"));
    
    let p = BeanDefinitionPropertiesCodeGenerator::new();
    p.register("p".to_string(), "v".to_string());
    assert_eq!(p.get("p"), Some("v".to_string()));
    
    let c = BeanFactoryInitializationAotContribution::new();
    c.register("c".to_string(), "v".to_string());
    assert_eq!(c.get("c"), Some("v".to_string()));
    
    let code = BeanFactoryInitializationCode::new();
    code.register("code".to_string(), "v".to_string());
    assert_eq!(code.get("code"), Some("v".to_string()));
    
    let filter = BeanRegistrationExcludeFilter::new();
    filter.register("f".to_string(), "criteria".to_string());
    assert!(filter.contains("f"));
    
    let df = DefaultBeanRegistrationCodeFragments::new();
    df.register("d".to_string(), "v".to_string());
    assert_eq!(df.count(), 1);
    
    let dec = BeanRegistrationCodeFragmentsDecorator::new();
    dec.register("d".to_string(), "v".to_string());
    assert!(dec.contains("d"));
    
    let del = BeanDefinitionPropertyValueCodeGeneratorDelegates::new();
    del.register("d".to_string(), "v".to_string());
    assert_eq!(del.count(), 1);
    
    let g = InstanceSupplierCodeGenerator::new();
    g.register("i".to_string(), "v".to_string());
    assert_eq!(g.get("i"), Some("v".to_string()));
}

// ═══════════════════════════════════════════════════════════════════
// AbstractBeanFactory Spring 语义测试（重写后的真实实现）
// ═══════════════════════════════════════════════════════════════════

#[test]
fn abstract_bean_factory_full_lifecycle() {
    use std::sync::Arc;
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    let f = AbstractBeanFactory::new();
    
    // 注册 Bean 定义
    let def: Arc<dyn std::any::Any + Send + Sync> = Arc::new(42i32);
    f.registerBeanDefinition("myBean".to_string(), def);
    assert!(f.containsBeanDefinition("myBean"));
    assert_eq!(f.getBeanDefinitionCount(), 1);
    
    // 注册单例
    let instance: Arc<dyn std::any::Any + Send + Sync> = Arc::new(100i32);
    f.registerSingleton("singleton1".to_string(), instance);
    assert!(f.containsSingleton("singleton1"));
    
    // 完整 bean 存在性检查
    assert!(f.containsBean("myBean"));
    assert!(f.containsBean("singleton1"));
    assert!(!f.containsBean("nonexistent"));
    
    // 别名管理
    f.registerAlias("alias1".to_string(), "myBean".to_string()).unwrap();
    assert_eq!(f.getAliasCount(), 1);
    assert_eq!(f.resolveAlias("alias1"), "myBean");
    
    // 父子工厂层级
    f.setParentBeanFactory("parent".to_string());
    assert_eq!(f.getParentBeanFactory(), Some("parent".to_string()));
    
    // 循环依赖检测
    assert!(!f.isBeanInCreation("bean1"));
    f.markBeanAsInCreation("bean1");
    assert!(f.isBeanInCreation("bean1"));
    f.markBeanAsCreated("bean1");
    assert!(!f.isBeanInCreation("bean1"));
    
    // 本地 Bean 计数
    assert_eq!(f.getLocalBeanCount(), 2);
}

#[test]
fn abstract_bean_factory_type_lookup() {
    use std::any::TypeId;
    use std::sync::Arc;
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    let f = AbstractBeanFactory::new();
    f.registerBeanDefinition("bean1".to_string(), Arc::new(1) as Arc<dyn std::any::Any + Send + Sync>);
    f.registerTypeMapping(TypeId::of::<i32>(), "bean1".to_string());
    let names = f.getBeanNamesForType(TypeId::of::<i32>());
    assert_eq!(names, vec!["bean1"]);
    assert_eq!(f.getTypeMappingCount(), 1);
}

#[test]
fn abstract_bean_factory_scope_registration() {
    use vernal_beans::ScopeKey;
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    let f = AbstractBeanFactory::new();
    assert_eq!(f.getRegisteredScopeCount(), 0);
    f.registerScope("session".to_string(), ScopeKey::of::<String>());
    f.registerScope("request".to_string(), ScopeKey::of::<i32>());
    assert_eq!(f.getRegisteredScopeCount(), 2);
    assert!(f.getRegisteredScope("session").is_some());
}

#[test]
fn abstract_bean_factory_destroy_singletons() {
    use std::sync::Arc;
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    let f = AbstractBeanFactory::new();
    f.registerSingleton("s1".to_string(), Arc::new(1) as Arc<dyn std::any::Any + Send + Sync>);
    f.registerSingleton("s2".to_string(), Arc::new(2) as Arc<dyn std::any::Any + Send + Sync>);
    assert_eq!(f.getSingletonCount(), 2);
    f.destroySingletons();
    assert_eq!(f.getSingletonCount(), 0);
}

#[test]
fn abstract_bean_factory_remove_bean_definition() {
    use std::sync::Arc;
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    let f = AbstractBeanFactory::new();
    f.registerBeanDefinition("bean1".to_string(), Arc::new(1) as Arc<dyn std::any::Any + Send + Sync>);
    assert!(f.removeBeanDefinition("bean1"));
    assert!(!f.containsBeanDefinition("bean1"));
    assert!(!f.removeBeanDefinition("bean1"));
}

// ═══════════════════════════════════════════════════════════════════
// DefaultListableBeanFactory Spring 语义测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn default_listable_bean_factory_register_and_query() {
    use std::sync::Arc;
    use vernal_beans::factory::support::default_listable_bean_factory::DefaultListableBeanFactory;
    let f = DefaultListableBeanFactory::new();
    let def: Arc<dyn std::any::Any + Send + Sync> = Arc::new("test".to_string());
    f.registerBeanDefinition("bean1".to_string(), def);
    assert!(f.containsBeanDefinition("bean1"));
    assert_eq!(f.getBeanDefinitionCount(), 1);
    let names = f.getBeanDefinitionNames();
    assert!(names.contains(&"bean1".to_string()));
}

#[test]
fn default_listable_bean_factory_pre_instantiate_freeze() {
    use std::sync::Arc;
    use vernal_beans::factory::support::default_listable_bean_factory::DefaultListableBeanFactory;
    let f = DefaultListableBeanFactory::new();
    f.registerBeanDefinition("bean1".to_string(), Arc::new(1) as Arc<dyn std::any::Any + Send + Sync>);
    f.registerBeanDefinition("bean2".to_string(), Arc::new(2) as Arc<dyn std::any::Any + Send + Sync>);
    assert!(!f.isConfigurationFrozen());
    assert!(f.preInstantiateSingletons().is_ok());
    assert!(f.isConfigurationFrozen());
}

#[test]
fn default_listable_bean_factory_ignore_dependency() {
    use std::any::TypeId;
    use vernal_beans::factory::support::default_listable_bean_factory::DefaultListableBeanFactory;
    let f = DefaultListableBeanFactory::new();
    assert_eq!(f.getIgnoredDependencyTypeCount(), 0);
    f.ignoreDependencyType(TypeId::of::<String>());
    assert!(f.isDependencyTypeIgnored(TypeId::of::<String>()));
    assert_eq!(f.getIgnoredDependencyTypeCount(), 1);
}

#[test]
fn default_listable_bean_factory_resolvable_dependency() {
    use std::any::TypeId;
    use std::sync::Arc;
    use vernal_beans::factory::support::default_listable_bean_factory::DefaultListableBeanFactory;
    let f = DefaultListableBeanFactory::new();
    let value: Arc<dyn std::any::Any + Send + Sync> = Arc::new(42i32);
    f.registerResolvableDependency(TypeId::of::<i32>(), value);
    let v = f.getResolvableDependency(TypeId::of::<i32>()).unwrap();
    assert_eq!(v.downcast_ref::<i32>().copied(), Some(42));
}

#[test]
fn default_listable_bean_factory_dependency_descriptor() {
    use std::any::TypeId;
    use vernal_beans::factory::support::default_listable_bean_factory::DefaultListableBeanFactory;
    use vernal_beans::factory::support::dependency_descriptor::DependencyDescriptor;
    let f = DefaultListableBeanFactory::new();
    let desc = DependencyDescriptor::new(TypeId::of::<String>(), "String".to_string(), true)
        .with_qualifier("primary".to_string());
    f.registerDependencyDescriptor("myBean".to_string(), desc);
    let retrieved = f.getDependencyDescriptor("myBean");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().qualifier, Some("primary".to_string()));
}

#[test]
fn default_listable_bean_factory_is_autowire_candidate() {
    use std::sync::Arc;
    use vernal_beans::factory::support::default_listable_bean_factory::DefaultListableBeanFactory;
    let f = DefaultListableBeanFactory::new();
    f.registerBeanDefinition("candidate".to_string(), Arc::new(1) as Arc<dyn std::any::Any + Send + Sync>);
    assert!(f.isAutowireCandidate("candidate"));
    assert!(!f.isAutowireCandidate("nonexistent"));
}

#[test]
fn default_listable_bean_factory_remove() {
    use std::sync::Arc;
    use vernal_beans::factory::support::default_listable_bean_factory::DefaultListableBeanFactory;
    let f = DefaultListableBeanFactory::new();
    f.registerBeanDefinition("bean1".to_string(), Arc::new(1) as Arc<dyn std::any::Any + Send + Sync>);
    f.registerBeanDefinition("bean2".to_string(), Arc::new(2) as Arc<dyn std::any::Any + Send + Sync>);
    assert_eq!(f.getBeanDefinitionCount(), 2);
    f.removeBeanDefinition("bean1");
    assert_eq!(f.getBeanDefinitionCount(), 1);
    assert!(!f.containsBeanDefinition("bean1"));
    assert!(f.containsBeanDefinition("bean2"));
}

#[test]
fn default_listable_bean_factory_get_all_names() {
    use std::sync::Arc;
    use vernal_beans::factory::support::default_listable_bean_factory::DefaultListableBeanFactory;
    let f = DefaultListableBeanFactory::new();
    f.registerBeanDefinition("a".to_string(), Arc::new(1) as Arc<dyn std::any::Any + Send + Sync>);
    f.registerBeanDefinition("b".to_string(), Arc::new(2) as Arc<dyn std::any::Any + Send + Sync>);
    let all = f.getAllBeanNames();
    assert_eq!(all.len(), 2);
    assert!(all.contains(&"a".to_string()));
    assert!(all.contains(&"b".to_string()));
}

// ═══════════════════════════════════════════════════════════════════
// BeanDefinitionReader 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bean_definition_reader_resource() {
    use vernal_beans::factory::support::bean_definition_reader::Resource;
    let r = Resource::new("beans.xml".to_string(), "classpath:beans.xml".to_string());
    assert_eq!(r.name, "beans.xml");
    assert!(r.exists);
    assert!(r.description().contains("beans.xml"));
}

#[test]
fn bean_definition_reader_impl_complete() {
    use std::any::TypeId;
    use vernal_beans::factory::support::bean_definition_reader::{
        AbstractBeanDefinitionReaderImpl, BeanDefinitionReader,
    };
    let reader = AbstractBeanDefinitionReaderImpl::new("file:".to_string());
    assert_eq!(reader.get_bean_class_count(), 0);
    assert_eq!(reader.get_bean_definition_count(), 0);
    assert_eq!(reader.get_resource_loader_name(), "file:");
    assert_eq!(reader.get_class_loader_type_id(), None);
    reader.increment_bean_class_count();
    reader.increment_bean_class_count();
    reader.increment_bean_definition_count();
    reader.set_registry_size(5);
    reader.set_class_loader_type_id(TypeId::of::<String>());
    assert_eq!(reader.get_bean_class_count(), 2);
    assert_eq!(reader.get_bean_definition_count(), 1);
    assert_eq!(reader.get_registry_size(), 5);
    assert_eq!(reader.get_class_loader_type_id(), Some(TypeId::of::<String>()));
}

#[test]
fn bean_definition_reader_load_bean_definitions() {
    use vernal_beans::factory::support::bean_definition_reader::{
        AbstractBeanDefinitionReaderImpl, BeanDefinitionReader, Resource,
    };
    let reader = AbstractBeanDefinitionReaderImpl::new("classpath".to_string());
    let r = Resource::new("test.xml".to_string(), "classpath:test.xml".to_string());
    let result = reader.load_bean_definitions(&r);
    assert!(result.is_ok());
    let resources = vec![
        Resource::new("a.xml".to_string(), "classpath:a.xml".to_string()),
        Resource::new("b.xml".to_string(), "classpath:b.xml".to_string()),
    ];
    let batch_result = reader.load_bean_definitions_batch(&resources);
    assert!(batch_result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════
// AutowiredAnnotationBeanPostProcessor 完整语义测试（重写后）
// ═══════════════════════════════════════════════════════════════════

#[test]
fn autowired_bpp_register_field_with_qualifier() {
    use vernal_beans::factory::annotation::autowired_annotation_bean_post_processor::AutowiredAnnotationBeanPostProcessor;
    use vernal_beans::Qualifier;
    let pp = AutowiredAnnotationBeanPostProcessor::new();
    let q = Qualifier::new("primary").unwrap();
    pp.register_field_with_qualifier(
        TypeId::of::<i32>(),
        "count".to_string(),
        TypeId::of::<i32>(),
        q,
        false,  // optional
    );
    let points = pp.get_field_injection_points(TypeId::of::<i32>());
    assert_eq!(points.len(), 1);
    assert_eq!(points[0].member_name, "count");
    assert!(!points[0].required);
    assert!(points[0].qualifier.is_some());
}

#[test]
fn autowired_bpp_parse_annotation() {
    use vernal_beans::factory::annotation::autowired_annotation_bean_post_processor::AutowiredAnnotationBeanPostProcessor;
    use std::sync::Arc;
    let pp = AutowiredAnnotationBeanPostProcessor::new();
    let type_id = TypeId::of::<i32>();
    pp.register_field(type_id, "field_a".to_string(), TypeId::of::<String>());
    pp.register_field(type_id, "field_b".to_string(), TypeId::of::<String>());
    pp.register_method(type_id, "setDep".to_string(), TypeId::of::<String>());
    
    let points = pp.parse_annotation(type_id);
    assert_eq!(points.len(), 3);
}

#[test]
fn autowired_bpp_find_dependency() {
    use std::sync::Arc;
    use vernal_beans::factory::annotation::autowired_annotation_bean_post_processor::{
        AutowiredAnnotationBeanPostProcessor, InjectionPoint,
    };
    let pp = AutowiredAnnotationBeanPostProcessor::new();
    let mut available = std::collections::HashMap::new();
    available.insert(
        TypeId::of::<i32>(),
        Arc::new(42i32) as Arc<dyn std::any::Any + Send + Sync>,
    );
    let point = InjectionPoint::new("count".to_string(), TypeId::of::<i32>());
    let dep = pp.find_dependency(&point, &available);
    assert!(dep.is_some());
    assert_eq!(dep.unwrap().downcast_ref::<i32>().copied(), Some(42));
    
    // 找不到的依赖
    let missing_point = InjectionPoint::new("missing".to_string(), TypeId::of::<String>());
    assert!(pp.find_dependency(&missing_point, &available).is_none());
}

#[test]
fn autowired_bpp_perform_injection_success() {
    use std::sync::Arc;
    use std::collections::HashMap;
    use vernal_beans::factory::annotation::autowired_annotation_bean_post_processor::AutowiredAnnotationBeanPostProcessor;
    let pp = AutowiredAnnotationBeanPostProcessor::new();
    let type_id = TypeId::of::<i32>();
    pp.register_field(type_id, "field_a".to_string(), TypeId::of::<String>());
    pp.register_field(type_id, "field_b".to_string(), TypeId::of::<String>());
    pp.register_method(type_id, "setDep".to_string(), TypeId::of::<String>());
    
    let mut available: HashMap<TypeId, Arc<dyn std::any::Any + Send + Sync>> = HashMap::new();
    available.insert(TypeId::of::<String>(), Arc::new("hello".to_string()));
    
    let result = pp.perform_injection(type_id, &available);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3);  // 注入 3 个依赖
    assert_eq!(pp.injection_count(), 3);
}

#[test]
fn autowired_bpp_perform_injection_required_missing() {
    use std::sync::Arc;
    use std::collections::HashMap;
    use vernal_beans::factory::annotation::autowired_annotation_bean_post_processor::AutowiredAnnotationBeanPostProcessor;
    let pp = AutowiredAnnotationBeanPostProcessor::new();
    let type_id = TypeId::of::<i32>();
    // 注册一个必需但找不到的依赖
    pp.register_field(type_id, "missing_dep".to_string(), TypeId::of::<String>());
    
    let available: HashMap<TypeId, Arc<dyn std::any::Any + Send + Sync>> = HashMap::new();
    
    let result = pp.perform_injection(type_id, &available);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Required dependency not found"));
    assert!(err.contains("missing_dep"));
}

#[test]
fn autowired_bpp_perform_injection_optional_missing_ok() {
    use std::sync::Arc;
    use std::collections::HashMap;
    use vernal_beans::Qualifier;
    use vernal_beans::factory::annotation::autowired_annotation_bean_post_processor::AutowiredAnnotationBeanPostProcessor;
    let pp = AutowiredAnnotationBeanPostProcessor::new();
    let type_id = TypeId::of::<i32>();
    // 注册一个可选的缺失依赖
    let q = Qualifier::new("optional").unwrap();
    pp.register_field_with_qualifier(
        type_id,
        "optional_dep".to_string(),
        TypeId::of::<String>(),
        q,
        false,  // optional
    );
    
    let available: HashMap<TypeId, Arc<dyn std::any::Any + Send + Sync>> = HashMap::new();
    
    let result = pp.perform_injection(type_id, &available);
    // 可选依赖缺失不应该失败
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);  // 注入了 0 个（找不到可选依赖）
}

#[test]
fn autowired_bpp_process_injection_full() {
    use std::sync::Arc;
    use std::collections::HashMap;
    use vernal_beans::factory::annotation::autowired_annotation_bean_post_processor::AutowiredAnnotationBeanPostProcessor;
    let pp = AutowiredAnnotationBeanPostProcessor::new();
    let type_id = TypeId::of::<i32>();
    pp.register_field(type_id, "name".to_string(), TypeId::of::<String>());
    
    let mut available: HashMap<TypeId, Arc<dyn std::any::Any + Send + Sync>> = HashMap::new();
    available.insert(TypeId::of::<String>(), Arc::new("Alice".to_string()));
    
    let bean: Arc<dyn std::any::Any + Send + Sync> = Arc::new(42i32);
    let result = pp.process_injection(type_id, bean, &available);
    assert!(result.is_ok());
    assert!(pp.is_initialized());
    assert_eq!(pp.injection_count(), 1);
}

#[test]
fn autowired_bpp_injection_count_reset() {
    use vernal_beans::factory::annotation::autowired_annotation_bean_post_processor::AutowiredAnnotationBeanPostProcessor;
    let pp = AutowiredAnnotationBeanPostProcessor::new();
    pp.register_field(TypeId::of::<i32>(), "field".to_string(), TypeId::of::<String>());
    
    let mut available = std::collections::HashMap::new();
    available.insert(
        TypeId::of::<String>(),
        Arc::new("test".to_string()) as Arc<dyn std::any::Any + Send + Sync>,
    );
    let _ = pp.perform_injection(TypeId::of::<i32>(), &available);
    assert!(pp.injection_count() > 0);
    
    pp.reset_injection_count();
    assert_eq!(pp.injection_count(), 0);
}

#[test]
fn autowired_bpp_initialize_lifecycle() {
    use vernal_beans::factory::annotation::autowired_annotation_bean_post_processor::AutowiredAnnotationBeanPostProcessor;
    let pp = AutowiredAnnotationBeanPostProcessor::new();
    assert!(!pp.is_initialized());
    pp.initialize();
    assert!(pp.is_initialized());
}
