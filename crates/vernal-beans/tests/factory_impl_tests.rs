//! 针对 factory/support/、factory/annotation/、factory/aot/ 新增实现的针对性测试。
use std::any::TypeId;

// ═══════════════════════════════════════════════════════════════════
// factory/annotation/ 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn autowired_annotation_bean_post_processor_register_field() {
    use vernal_beans::factory::annotation::autowired_annotation_bean_post_processor::AutowiredAnnotationBeanPostProcessor;
    let pp = AutowiredAnnotationBeanPostProcessor::new();
    pp.register_field(TypeId::of::<String>(), "name".to_string(), TypeId::of::<String>());
    let points = pp.get_field_injection_points(TypeId::of::<String>());
    assert_eq!(points, vec!["name"]);
}

#[test]
fn autowired_annotation_bean_post_processor_register_method() {
    use vernal_beans::factory::annotation::autowired_annotation_bean_post_processor::AutowiredAnnotationBeanPostProcessor;
    let pp = AutowiredAnnotationBeanPostProcessor::new();
    pp.register_method(TypeId::of::<i32>(), "setValue".to_string(), TypeId::of::<i32>());
    let points = pp.get_method_injection_points(TypeId::of::<i32>());
    assert_eq!(points, vec!["setValue"]);
}

#[test]
fn autowired_annotation_bean_post_processor_initialize() {
    use vernal_beans::factory::annotation::autowired_annotation_bean_post_processor::AutowiredAnnotationBeanPostProcessor;
    let pp = AutowiredAnnotationBeanPostProcessor::new();
    assert!(!pp.is_initialized());
    pp.initialize();
    assert!(pp.is_initialized());
}

#[test]
fn autowired_annotation_bean_post_processor_injection_count() {
    use vernal_beans::factory::annotation::autowired_annotation_bean_post_processor::AutowiredAnnotationBeanPostProcessor;
    let pp = AutowiredAnnotationBeanPostProcessor::new();
    pp.register_field(TypeId::of::<String>(), "name".to_string(), TypeId::of::<String>());
    pp.register_method(TypeId::of::<String>(), "setName".to_string(), TypeId::of::<String>());
    assert_eq!(pp.injection_count(TypeId::of::<String>()), 2);
}

#[test]
fn init_destroy_annotation_bean_post_processor_register() {
    use vernal_beans::factory::annotation::init_destroy_annotation_bean_post_processor::InitDestroyAnnotationBeanPostProcessor;
    let pp = InitDestroyAnnotationBeanPostProcessor::new();
    pp.register_init_method("init".to_string());
    pp.register_destroy_method("destroy".to_string());
    assert_eq!(pp.init_method_count(), 1);
    assert_eq!(pp.destroy_method_count(), 1);
    assert!(pp.has_init_method("init"));
    assert!(pp.has_destroy_method("destroy"));
}

#[test]
fn init_destroy_annotation_bean_post_processor_multiple() {
    use vernal_beans::factory::annotation::init_destroy_annotation_bean_post_processor::InitDestroyAnnotationBeanPostProcessor;
    let pp = InitDestroyAnnotationBeanPostProcessor::new();
    pp.register_init_method("init1".to_string());
    pp.register_init_method("init2".to_string());
    pp.register_destroy_method("destroy1".to_string());
    assert_eq!(pp.init_method_count(), 2);
    assert_eq!(pp.destroy_method_count(), 1);
}

#[test]
fn qualifier_annotation_resolver_register() {
    use vernal_beans::factory::annotation::qualifier_annotation_autowire_candidate_resolver::QualifierAnnotationAutowireCandidateResolver;
    let r = QualifierAnnotationAutowireCandidateResolver::new();
    r.register_qualifier(TypeId::of::<String>(), "primary".to_string());
    assert!(r.has_qualifier(TypeId::of::<String>(), "primary"));
    assert_eq!(r.qualifier_count(TypeId::of::<String>()), 1);
}

#[test]
fn qualifier_annotation_resolver_multiple() {
    use vernal_beans::factory::annotation::qualifier_annotation_autowire_candidate_resolver::QualifierAnnotationAutowireCandidateResolver;
    let r = QualifierAnnotationAutowireCandidateResolver::new();
    r.register_qualifier(TypeId::of::<String>(), "primary".to_string());
    r.register_qualifier(TypeId::of::<String>(), "secondary".to_string());
    let qualifiers = r.get_qualifiers(TypeId::of::<String>());
    assert_eq!(qualifiers.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════
// factory/aot/ 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bean_registration_aot_processor_process() {
    use vernal_beans::factory::aot::bean_registration_aot_processor::BeanRegistrationAotProcessor;
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    let p = BeanRegistrationAotProcessor::new();
    let def = "MyBean".to_string();
    assert!(p.process(TypeId::of::<String>(), "MyBean".to_string()).is_ok());
    assert_eq!(p.processed_count(), 1);
    assert!(p.contains(TypeId::of::<String>()));
}

#[test]
fn bean_registration_aot_processor_clear() {
    use vernal_beans::factory::aot::bean_registration_aot_processor::BeanRegistrationAotProcessor;
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    let p = BeanRegistrationAotProcessor::new();
    p.process(TypeId::of::<String>(), "MyBean".to_string());
    p.process(TypeId::of::<i32>(), "Other".to_string());
    assert_eq!(p.processed_count(), 2);
    p.clear();
    assert_eq!(p.processed_count(), 0);
}

#[test]
fn bean_factory_initialization_aot_processor_steps() {
    use vernal_beans::factory::aot::bean_factory_initialization_aot_processor::BeanFactoryInitializationAotProcessor;
    let p = BeanFactoryInitializationAotProcessor::new();
    assert!(!p.is_initialized());
    p.process_step("step1");
    p.process_step("step2");
    assert_eq!(p.step_count(), 2);
    assert!(p.has_step("step1"));
    p.mark_initialized();
    assert!(p.is_initialized());
}

#[test]
fn bean_factory_initialization_aot_processor_clear_steps() {
    use vernal_beans::factory::aot::bean_factory_initialization_aot_processor::BeanFactoryInitializationAotProcessor;
    let p = BeanFactoryInitializationAotProcessor::new();
    p.process_step("a");
    p.process_step("b");
    assert_eq!(p.step_count(), 2);
    p.clear_steps();
    assert_eq!(p.step_count(), 0);
}

#[test]
fn autowired_arguments_basic() {
    use vernal_beans::factory::aot::autowired_arguments::AutowiredArguments;
    let args = AutowiredArguments::empty();
    assert!(args.is_empty());
    assert_eq!(args.count(), 0);
}

#[test]
fn autowired_arguments_from_arguments() {
    use std::sync::Arc;
    use vernal_beans::factory::aot::autowired_arguments::AutowiredArguments;
    let args_vec: Vec<Arc<dyn std::any::Any + Send + Sync>> = vec![Arc::new(42i32)];
    let args = AutowiredArguments::from_arguments(args_vec);
    assert!(!args.is_empty());
    assert_eq!(args.count(), 1);
    assert_eq!(args.type_id_at(0), Some(TypeId::of::<i32>()));
}

// ═══════════════════════════════════════════════════════════════════
// factory/support/ 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn abstract_bean_factory_register_definition() {
    use std::sync::Arc;
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    let f = AbstractBeanFactory::new();
    f.register_bean_definition("myBean".to_string(), Arc::new(42i32));
    assert_eq!(f.bean_definition_count(), 1);
    assert!(f.contains_bean_definition("myBean"));
    assert_eq!(f.bean_definition_names(), vec!["myBean"]);
}

#[test]
fn abstract_bean_factory_register_singleton() {
    use std::sync::Arc;
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    let f = AbstractBeanFactory::new();
    f.register_singleton("s1".to_string(), Arc::new("value".to_string()));
    assert_eq!(f.singleton_count(), 1);
    assert!(f.contains_singleton("s1"));
    let val = f.get_singleton("s1").unwrap();
    assert_eq!(val.downcast_ref::<String>().unwrap(), "value");
}

#[test]
fn abstract_bean_factory_register_alias() {
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    let f = AbstractBeanFactory::new();
    assert!(f.register_alias("alias1".to_string(), "myBean".to_string()).is_ok());
    assert_eq!(f.resolve_alias("alias1"), "myBean");
    assert_eq!(f.alias_count(), 1);
}

#[test]
fn abstract_bean_factory_alias_conflict() {
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    let f = AbstractBeanFactory::new();
    f.register_alias("alias1".to_string(), "bean1".to_string()).unwrap();
    // 重复注册别名到不同 bean 应该失败
    assert!(f.register_alias("alias1".to_string(), "bean2".to_string()).is_err());
    // 重复注册到相同 bean 应该成功
    assert!(f.register_alias("alias1".to_string(), "bean1".to_string()).is_ok());
}

#[test]
fn abstract_bean_factory_freeze() {
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    let f = AbstractBeanFactory::new();
    assert!(!f.is_configuration_frozen());
    f.freeze_configuration();
    assert!(f.is_configuration_frozen());
}

#[test]
fn abstract_bean_factory_destroy_singletons() {
    use std::sync::Arc;
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    let f = AbstractBeanFactory::new();
    f.register_singleton("s1".to_string(), Arc::new(1i32));
    f.register_singleton("s2".to_string(), Arc::new(2i32));
    assert_eq!(f.singleton_count(), 2);
    f.destroy_singletons();
    assert_eq!(f.singleton_count(), 0);
}

#[test]
fn abstract_bean_factory_parent() {
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    use vernal_beans::ScopeKey;
    let f = AbstractBeanFactory::new();
    assert!(f.parent().is_none());
    f.set_parent(Some("parent1".to_string()));
    assert_eq!(f.parent(), Some("parent1".to_string()));
    let _ = ScopeKey::of::<String>();
}

#[test]
fn default_listable_bean_factory_type_mapping() {
    use vernal_beans::factory::support::default_listable_bean_factory::DefaultListableBeanFactory;
    let f = DefaultListableBeanFactory::new();
    f.register_type_mapping(TypeId::of::<String>(), "bean1".to_string());
    f.register_type_mapping(TypeId::of::<String>(), "bean2".to_string());
    f.register_type_mapping(TypeId::of::<i32>(), "bean3".to_string());
    let string_beans = f.get_bean_names_for_type(TypeId::of::<String>());
    assert_eq!(string_beans.len(), 2);
    assert_eq!(f.type_mapping_count(), 2);
}

#[test]
fn default_listable_bean_factory_dependency_descriptor() {
    use vernal_beans::factory::support::default_listable_bean_factory::DefaultListableBeanFactory;
    use vernal_beans::factory::support::dependency_descriptor::DependencyDescriptor;
    let f = DefaultListableBeanFactory::new();
    let desc = DependencyDescriptor::new(TypeId::of::<String>(), "String".to_string(), true)
        .with_qualifier("primary".to_string())
        .with_injection_point_name("name".to_string());
    f.register_dependency_descriptor("myBean".to_string(), desc);
    let retrieved = f.get_dependency_descriptor("myBean");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().qualifier, Some("primary".to_string()));
}

#[test]
fn dependency_descriptor_builder() {
    use vernal_beans::factory::support::dependency_descriptor::DependencyDescriptor;
    let desc = DependencyDescriptor::new(TypeId::of::<String>(), "String".to_string(), true);
    assert!(desc.is_required());
    assert!(!desc.has_qualifier());
    let desc2 = desc.with_qualifier("primary".to_string());
    assert!(desc2.has_qualifier());
}

#[test]
fn abstract_bean_factory_scope() {
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    use vernal_beans::ScopeKey;
    let f = AbstractBeanFactory::new();
    f.register_scope("session".to_string(), ScopeKey::of::<String>());
    f.register_scope("request".to_string(), ScopeKey::of::<i32>());
    assert_eq!(f.registered_scope_count(), 2);
    assert!(f.contains_scope("session"));
}

// ═══════════════════════════════════════════════════════════════════
// 顶层缺失文件测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bean_instantiation_exception() {
    use vernal_beans::bean_instantiation_exception::BeanInstantiationException;
    let e = BeanInstantiationException::new("failed to create");
    assert_eq!(e.message(), "failed to create");
}

#[test]
fn fatal_bean_exception() {
    use vernal_beans::fatal_bean_exception::FatalBeanException;
    let e = FatalBeanException::new("fatal error");
    assert_eq!(e.message(), "fatal error");
}

#[test]
fn invalid_property_exception() {
    use vernal_beans::invalid_property_exception::InvalidPropertyException;
    let e = InvalidPropertyException::new("invalid property");
    assert_eq!(e.message(), "invalid property");
}

#[test]
fn not_readable_property_exception() {
    use vernal_beans::not_readable_property_exception::NotReadablePropertyException;
    let e = NotReadablePropertyException::new("not readable");
    assert_eq!(e.message(), "not readable");
}

#[test]
fn not_writable_property_exception() {
    use vernal_beans::not_writable_property_exception::NotWritablePropertyException;
    let e = NotWritablePropertyException::new("not writable");
    assert_eq!(e.message(), "not writable");
}

#[test]
fn null_value_in_nested_path_exception() {
    use vernal_beans::null_value_in_nested_path_exception::NullValueInNestedPathException;
    let e = NullValueInNestedPathException::new("null in path");
    assert_eq!(e.message(), "null in path");
}

#[test]
fn property_access_exception() {
    use vernal_beans::property_access_exception::PropertyAccessException;
    let e = PropertyAccessException::new("access denied");
    assert_eq!(e.message(), "access denied");
}
