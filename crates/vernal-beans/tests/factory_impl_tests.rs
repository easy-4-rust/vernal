//! Tests for factory/support, factory/annotation, factory/aot implementations.
use std::any::TypeId;
use std::sync::Arc;

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
    use vernal_beans::factory::support::default_singleton_bean_registry::DefaultSingletonBeanRegistry;
    let r = DefaultSingletonBeanRegistry::new();
    assert_eq!(r.singleton_count(), 0);
    r.register_singleton("b1".to_string(), Arc::new(42i32));
    assert_eq!(r.singleton_count(), 1);
    assert!(r.contains_singleton("b1"));
}

#[test]
fn default_singleton_registry_l3() {
    use vernal_beans::factory::support::default_singleton_bean_registry::DefaultSingletonBeanRegistry;
    let r = DefaultSingletonBeanRegistry::new();
    r.add_singleton_factory(
        "b1".to_string(),
        Arc::new(|| Arc::new(99i32) as Arc<dyn std::any::Any + Send + Sync>),
    );
    let early = r.get_early_bean_reference("b1");
    assert!(early.is_some());
}

#[test]
fn default_singleton_registry_lifecycle() {
    use vernal_beans::factory::support::default_singleton_bean_registry::DefaultSingletonBeanRegistry;
    let r = DefaultSingletonBeanRegistry::new();
    assert!(!r.is_currently_in_creation("b1"));
    r.mark_as_in_creation("b1");
    assert!(r.is_currently_in_creation("b1"));
}

#[test]
fn bean_definition_defaults_test() {
    use vernal_beans::factory::support::bean_definition_defaults::BeanDefinitionDefaults;
    let mut d = BeanDefinitionDefaults::new();
    assert!(!d.lazy_init);
    d.set_lazy_init(true);
    assert!(d.lazy_init);
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
    let mut r = AbstractBeanDefinitionReader::new("classpath");
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
    use vernal_beans::factory::support::disposable_bean_adapter::DisposableBeanAdapter;
    let adapter = DisposableBeanAdapter::new("myBean".to_string(), Arc::new(42i32));
    assert_eq!(adapter.bean_name(), "myBean");
}

#[test]
fn managed_collections_test() {
    use vernal_beans::factory::support::managed_array::ManagedArray;
    use vernal_beans::factory::support::managed_list::ManagedList;
    use vernal_beans::factory::support::managed_map::ManagedMap;
    use vernal_beans::factory::support::managed_properties::ManagedProperties;
    use vernal_beans::factory::support::managed_set::ManagedSet;

    let list = ManagedList::new();
    list.add(Arc::new(1i32) as Arc<dyn std::any::Any + Send + Sync>);
    list.add(Arc::new(2i32) as Arc<dyn std::any::Any + Send + Sync>);
    assert_eq!(list.len(), 2);

    let map = ManagedMap::new();
    map.put(
        "k1".to_string(),
        Arc::new(1) as Arc<dyn std::any::Any + Send + Sync>,
    );
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
    use vernal_beans::factory::annotation::configurable::Configurable;
    use vernal_beans::factory::annotation::lookup::Lookup;
    use vernal_beans::factory::annotation::value::Value;

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
    let d = ParameterResolutionDelegate::new();
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
    h.reset();
    assert_eq!(h.registered_count(), 0);
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
    use vernal_beans::factory::aot::autowired_element_resolver::AutowiredElementResolver;
    use vernal_beans::factory::aot::autowired_field_value_resolver::AutowiredFieldValueResolver;
    use vernal_beans::factory::aot::autowired_method_arguments_resolver::AutowiredMethodArgumentsResolver;
    use vernal_beans::factory::aot::bean_definition_method_generator::BeanDefinitionMethodGenerator;
    use vernal_beans::factory::aot::bean_definition_method_generator_factory::BeanDefinitionMethodGeneratorFactory;
    use vernal_beans::factory::aot::bean_definition_properties_code_generator::BeanDefinitionPropertiesCodeGenerator;
    use vernal_beans::factory::aot::bean_definition_property_value_code_generator_delegates::BeanDefinitionPropertyValueCodeGeneratorDelegates;
    use vernal_beans::factory::aot::bean_factory_initialization_aot_contribution::BeanFactoryInitializationAotContribution;
    use vernal_beans::factory::aot::bean_factory_initialization_code::BeanFactoryInitializationCode;
    use vernal_beans::factory::aot::bean_instance_supplier::BeanInstanceSupplier;
    use vernal_beans::factory::aot::bean_registration_code_fragments_decorator::BeanRegistrationCodeFragmentsDecorator;
    use vernal_beans::factory::aot::bean_registration_exclude_filter::BeanRegistrationExcludeFilter;
    use vernal_beans::factory::aot::bean_registrations_aot_contribution::BeanRegistrationsAotContribution;
    use vernal_beans::factory::aot::bean_registrations_aot_processor::BeanRegistrationsAotProcessor;
    use vernal_beans::factory::aot::bean_registrations_code::BeanRegistrationsCode;
    use vernal_beans::factory::aot::default_bean_registration_code_fragments::DefaultBeanRegistrationCodeFragments;
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
