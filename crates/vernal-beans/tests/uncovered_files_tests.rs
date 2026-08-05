//! 测试未覆盖的核心文件方法。
use std::sync::Arc;

// ═══════════════════════════════════════════════════════════════════
// BeanFactoryUtils 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bean_factory_utils_transformed_bean_name() {
    use vernal_beans::BeanFactoryUtils;
    assert_eq!(BeanFactoryUtils::transformed_bean_name("&myBean"), "myBean");
    assert_eq!(BeanFactoryUtils::transformed_bean_name("myBean"), "myBean");
}

#[test]
fn bean_factory_utils_is_factory_bean() {
    use vernal_beans::BeanFactoryUtils;
    assert!(BeanFactoryUtils::is_factory_bean("&myBean"));
    assert!(!BeanFactoryUtils::is_factory_bean("myBean"));
}

#[test]
fn bean_factory_utils_check_is_factory_bean() {
    use vernal_beans::BeanFactoryUtils;
    assert!(BeanFactoryUtils::check_is_factory_bean("&myBean"));
    assert!(!BeanFactoryUtils::check_is_factory_bean("myBean"));
}

// ═══════════════════════════════════════════════════════════════════
// BeanDescCache 测试
// ═══════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════
// DestructibleBeanAdapter 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn destructible_bean_adapter_basic() {
    use vernal_beans::destructible_bean_adapter::DisposableBeanAdapter;
    let adapter = DisposableBeanAdapter::new("myBean".to_string(), Arc::new(42i32), None);
    assert_eq!(adapter.bean_name(), "myBean");
    let bean = adapter.bean();
    assert_eq!(bean.downcast_ref::<i32>().copied(), Some(42));
}

#[test]
fn destructible_bean_adapter_destroy() {
    use vernal_beans::destructible_bean_adapter::DisposableBeanAdapter;
    let adapter = DisposableBeanAdapter::new("myBean".to_string(), Arc::new(42i32), None);
    assert!(adapter.destroy().is_ok());
}

// ═══════════════════════════════════════════════════════════════════
// ApplicationScope 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn application_scope_basic() {
    use vernal_beans::application_scope::ApplicationScope;
    let scope = ApplicationScope::new();
    assert_eq!(scope.cached_count(), 0);
}

#[test]
fn application_scope_destroy() {
    use vernal_beans::application_scope::ApplicationScope;
    let scope = ApplicationScope::new();
    scope.destroy();
    assert_eq!(scope.cached_count(), 0);
}

// ═══════════════════════════════════════════════════════════════════
// BeanDefinitionUtils 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bean_definition_utils_generate_bean_name() {
    use vernal_beans::bean_definition_utils;

    let name = bean_definition_utils::generate_bean_name(
        Some("com.example.MyService"),
        &vernal_beans::SimpleBeanDefinitionRegistry::new(),
    );
    assert!(!name.is_empty());
}

// ═══════════════════════════════════════════════════════════════════
// Exception 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bean_definition_override_exception() {
    use vernal_beans::BeanDefinitionOverrideException;
    let e = BeanDefinitionOverrideException::new("duplicate bean");
    assert_eq!(e.message(), "duplicate bean");
}

#[test]
fn bean_definition_parsing_exception() {
    use vernal_beans::BeanDefinitionParsingException;
    let e = BeanDefinitionParsingException::new("parse error");
    assert_eq!(e.message(), "parse error");
}

#[test]
fn bean_definition_validation_exception() {
    use vernal_beans::BeanDefinitionValidationException;
    let e = BeanDefinitionValidationException::new("validation error");
    assert_eq!(e.message(), "validation error");
}

#[test]
fn bean_instantiation_exception() {
    use vernal_beans::bean_instantiation_exception::BeanInstantiationException;
    let e = BeanInstantiationException::new("instantiation error");
    assert_eq!(e.message(), "instantiation error");
}

#[test]
fn bean_is_abstract_exception() {
    use vernal_beans::BeanIsAbstractException;
    let e = BeanIsAbstractException::new("abstract bean");
    assert_eq!(e.message(), "abstract bean");
}

#[test]
fn bean_is_not_a_factory_exception() {
    use vernal_beans::BeanIsNotAFactoryException;
    let e = BeanIsNotAFactoryException::new("not a factory");
    assert_eq!(e.message(), "not a factory");
}

#[test]
fn conversion_not_supported_exception() {
    use vernal_beans::conversion_not_supported_exception::ConversionNotSupportedException;
    let e = ConversionNotSupportedException::new("conversion not supported");
    assert_eq!(e.message(), "conversion not supported");
}

#[test]
fn type_mismatch_exception() {
    use vernal_beans::type_mismatch_exception::TypeMismatchException;
    let e = TypeMismatchException::new("type mismatch");
    assert_eq!(e.message(), "type mismatch");
}

#[test]
fn scope_not_active_exception() {
    use vernal_beans::ScopeNotActiveException;
    let e = ScopeNotActiveException::new("scope not active");
    assert_eq!(e.message(), "scope not active");
}

#[test]
fn property_batch_update_exception() {
    use vernal_beans::property_batch_update_exception::PropertyBatchUpdateException;
    let e = PropertyBatchUpdateException::new("batch update error");
    assert_eq!(e.message(), "batch update error");
}

#[test]
fn xml_bean_definition_store_exception() {
    use vernal_beans::XmlBeanDefinitionStoreException;
    let e = XmlBeanDefinitionStoreException::new("xml store error");
    assert_eq!(e.message(), "xml store error");
}

#[test]
fn aot_processing_exception() {
    use vernal_beans::AotProcessingException;
    let e = AotProcessingException::new("aot processing error");
    assert_eq!(e.message(), "aot processing error");
}

#[test]
fn aot_bean_processing_exception() {
    use vernal_beans::AotBeanProcessingException;
    let e = AotBeanProcessingException::new("aot bean error");
    assert_eq!(e.message(), "aot bean error");
}

#[test]
fn aot_exception() {
    use vernal_beans::AotException;
    let e = AotException::new("aot error");
    assert_eq!(e.message(), "aot error");
}

// ═══════════════════════════════════════════════════════════════════
// Factory/AOT 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bean_registration_aot_processor() {
    use vernal_beans::factory::aot::bean_registration_aot_processor::BeanRegistrationAotProcessor;
    let p = BeanRegistrationAotProcessor::new();
    let result = p.process(std::any::TypeId::of::<String>(), "MyBean".to_string());
    assert!(result.is_ok());
    assert_eq!(p.processed_count(), 1);
    assert!(p.contains(std::any::TypeId::of::<String>()));
}

#[test]
fn autowired_arguments() {
    use vernal_beans::factory::aot::autowired_arguments::AutowiredArguments;
    let args = AutowiredArguments::empty();
    assert!(args.is_empty());
    assert_eq!(args.count(), 0);
}

#[test]
fn autowired_arguments_from_vec() {
    use vernal_beans::factory::aot::autowired_arguments::AutowiredArguments;
    let args_vec: Vec<Arc<dyn std::any::Any + Send + Sync>> = vec![Arc::new(42i32)];
    let args = AutowiredArguments::from_arguments(args_vec);
    assert!(!args.is_empty());
    assert_eq!(args.count(), 1);
    assert_eq!(args.type_id_at(0), Some(std::any::TypeId::of::<i32>()));
}

#[test]
fn code_warnings() {
    use vernal_beans::factory::aot::code_warnings::CodeWarnings;
    let w = CodeWarnings::new();
    w.register("warning1".to_string(), "msg1".to_string());
    assert_eq!(w.count(), 1);
    assert_eq!(w.get("warning1"), Some("msg1".to_string()));
}

#[test]
fn bean_definition_method_generator() {
    use vernal_beans::factory::aot::bean_definition_method_generator::BeanDefinitionMethodGenerator;
    let g = BeanDefinitionMethodGenerator::new();
    g.register("create".to_string(), "body".to_string());
    assert_eq!(g.get("create"), Some("body".to_string()));
    assert_eq!(g.cache_size(), 1);
}

#[test]
fn bean_factory_initialization_aot_processor() {
    use vernal_beans::factory::aot::bean_factory_initialization_aot_processor::BeanFactoryInitializationAotProcessor;
    let p = BeanFactoryInitializationAotProcessor::new();
    assert!(!p.is_initialized());
    p.process_step("step1");
    assert_eq!(p.step_count(), 1);
    p.mark_initialized();
    assert!(p.is_initialized());
}

// ═══════════════════════════════════════════════════════════════════
// Factory/Annotation 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn qualifier_annotation_autowire_candidate_resolver() {
    use vernal_beans::factory::annotation::qualifier_annotation_autowire_candidate_resolver::QualifierAnnotationAutowireCandidateResolver;
    let r = QualifierAnnotationAutowireCandidateResolver::new();
    r.register_qualifier(std::any::TypeId::of::<String>(), "primary".to_string());
    assert!(r.has_qualifier(std::any::TypeId::of::<String>(), "primary"));
    assert_eq!(r.qualifier_count(std::any::TypeId::of::<String>()), 1);
}

#[test]
fn bean_factory_annotation_utils() {
    use vernal_beans::factory::annotation::bean_factory_annotation_utils::BeanFactoryAnnotationUtils;
    let u = BeanFactoryAnnotationUtils::new();
    u.mark_processed("Autowired".to_string());
    assert!(u.is_processed("Autowired"));
    assert_eq!(u.processed_count(), 1);
}

#[test]
fn jakarta_annotations_runtime_hints() {
    use vernal_beans::factory::annotation::jakarta_annotations_runtime_hints::JakartaAnnotationsRuntimeHints;
    let h = JakartaAnnotationsRuntimeHints::new();
    h.register();
    assert_eq!(h.registered_count(), 1);
    h.reset();
    assert_eq!(h.registered_count(), 0);
}

#[test]
fn init_destroy_annotation_bean_post_processor() {
    use vernal_beans::factory::annotation::init_destroy_annotation_bean_post_processor::InitDestroyAnnotationBeanPostProcessor;
    let pp = InitDestroyAnnotationBeanPostProcessor::new();
    pp.register_init_method("init".to_string());
    pp.register_destroy_method("destroy".to_string());
    assert_eq!(pp.init_method_count(), 1);
    assert_eq!(pp.destroy_method_count(), 1);
}

// ═══════════════════════════════════════════════════════════════════
// TransientTracker 测试
// ═══════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════
// ScopeError 测试
// ═══════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════
// ConversionService 测试
// ═══════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════
// RegistryBuilder 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn registry_builder_empty() {
    let b = vernal_beans::RegistryBuilder::new();
    let reg = b.build().unwrap();
    let c = vernal_beans::Container::new(reg);
    let result: Result<Arc<String>, _> = c.resolve();
    assert!(result.is_err());
}

#[test]
fn registry_builder_multiple_types() {
    let mut b = vernal_beans::RegistryBuilder::new();
    let _ = b.register(vernal_beans::ComponentDefinition::singleton::<String, _>(
        |_| "hello".to_string(),
    ));
    let _ = b.register(vernal_beans::ComponentDefinition::singleton::<i32, _>(
        |_| 42i32,
    ));
    let c = vernal_beans::Container::new(b.build().unwrap());
    let s: Arc<String> = c.resolve().unwrap();
    let i: Arc<i32> = c.resolve().unwrap();
    assert_eq!(*s, "hello");
    assert_eq!(*i, 42);
}

#[test]
fn registry_builder_with_qualifier() {
    let mut b = vernal_beans::RegistryBuilder::new();
    let q = vernal_beans::Qualifier::new("primary").unwrap();
    let _ = b.register(
        vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "primary".to_string())
            .qualified(q.clone()),
    );
    let _ = b.register(vernal_beans::ComponentDefinition::singleton::<String, _>(
        |_| "default".to_string(),
    ));
    let c = vernal_beans::Container::new(b.build().unwrap());
    let val: Arc<String> = c.resolve_qualified(&q).unwrap();
    assert_eq!(*val, "primary");
}

// ═══════════════════════════════════════════════════════════════════
// RootBeanDefinition 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn root_bean_definition_basic() {
    use vernal_beans::BeanDefinition;
    use vernal_beans::RootBeanDefinition;
    let rbd = RootBeanDefinition::new();
    assert_eq!(rbd.scope(), vernal_beans::Scope::Singleton);
    assert!(!rbd.is_abstract());
    assert!(rbd.is_singleton());
    assert!(!rbd.is_prototype());
    assert!(!rbd.is_lazy_init());
    assert!(!rbd.is_primary());
}

#[test]
fn root_bean_definition_setters() {
    use vernal_beans::RootBeanDefinition;
    let mut rbd = RootBeanDefinition::new();
    rbd.set_bean_class_name("com.example.MyService");
    assert_eq!(rbd.bean_class_name(), "com.example.MyService");
    rbd.set_scope(vernal_beans::Scope::Transient);
    assert_eq!(rbd.scope(), vernal_beans::Scope::Transient);
    rbd.set_lazy_init(true);
    assert!(rbd.is_lazy_init());
    rbd.set_abstract(true);
    assert!(rbd.is_abstract());
    rbd.set_primary(true);
    assert!(rbd.is_primary());
}

// ═══════════════════════════════════════════════════════════════════
// GenericBeanDefinition 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn generic_bean_definition_basic() {
    use vernal_beans::GenericBeanDefinition;
    let gbd = GenericBeanDefinition::new();
    assert_eq!(gbd.scope(), vernal_beans::Scope::Singleton);
}

// ═══════════════════════════════════════════════════════════════════
// BeanDefinitionBuilder 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bean_definition_builder_generic() {
    use vernal_beans::BeanDefinitionBuilder;
    let def = BeanDefinitionBuilder::generic("com.example.Service")
        .set_scope(vernal_beans::Scope::Transient)
        .set_lazy_init(true)
        .build();
    assert!(def.is_lazy_init());
}

#[test]
fn bean_definition_builder_root() {
    use vernal_beans::BeanDefinitionBuilder;
    let def = BeanDefinitionBuilder::root("com.example.Root")
        .set_primary(true)
        .build();
    assert!(def.is_primary());
}

// ═══════════════════════════════════════════════════════════════════
// ConstructorArgumentValues 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn cav_basic() {
    use vernal_beans::ConstructorArgumentValues;
    use vernal_beans::factory::config::constructor_argument_values::ValueHolder;
    let mut cav = ConstructorArgumentValues::new();
    assert!(cav.is_empty());
    cav.add_indexed_argument_value(0, ValueHolder::new(Arc::new("first")));
    assert!(!cav.is_empty());
    assert!(cav.has_indexed_argument_value(0));
    assert!(!cav.has_indexed_argument_value(99));
}

// ═══════════════════════════════════════════════════════════════════
// Dependency 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn dependency_display() {
    let d1 = vernal_beans::Dependency::of::<String>();
    let d2 =
        vernal_beans::Dependency::qualified::<String>(vernal_beans::Qualifier::new("q").unwrap());
    let d3 = vernal_beans::Dependency::optional_of::<String>();
    let d4 = vernal_beans::Dependency::provider_of::<String>();
    let d5 = vernal_beans::Dependency::trait_of::<dyn std::fmt::Debug + Send + Sync>();
    assert!(!format!("{}", d1).is_empty());
    assert!(!format!("{}", d2).is_empty());
    assert!(!format!("{}", d3).is_empty());
    assert!(!format!("{}", d4).is_empty());
    assert!(!format!("{}", d5).is_empty());
}

// ═══════════════════════════════════════════════════════════════════
// ResolveError 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn resolve_error_display() {
    use vernal_beans::{ComponentKey, ResolveError, ScopeKey, TraitKey};
    let errors: Vec<ResolveError> = vec![
        ResolveError::NotFound {
            component: "t".into(),
            path: vec!["r".into()],
        },
        ResolveError::Ambiguous {
            component: "t".into(),
            candidates: vec!["a".into(), "b".into()],
            path: vec!["r".into()],
        },
        ResolveError::UndeclaredDependency {
            component: ComponentKey::of::<String>(),
            dependency: "d".into(),
        },
        ResolveError::TypeMismatch {
            component: ComponentKey::of::<String>(),
        },
        ResolveError::TraitBindingTypeMismatch {
            binding: TraitKey::of::<dyn std::fmt::Debug>(),
            target: ComponentKey::of::<i32>(),
        },
        ResolveError::Construction {
            component: ComponentKey::of::<String>(),
            source: Arc::new(std::io::Error::new(std::io::ErrorKind::Other, "e")),
        },
        ResolveError::CircularRuntime {
            path: vec!["a".into(), "b".into(), "c".into()],
        },
        ResolveError::ProviderUsedDuringConstruction {
            component: ComponentKey::of::<String>(),
            dependency: "d".into(),
        },
        ResolveError::ScopeNotActive {
            component: ComponentKey::of::<String>(),
            scope: ScopeKey::of::<String>(),
        },
        ResolveError::ScopeOwnerMismatch {
            scope: ScopeKey::of::<String>(),
        },
    ];
    for e in &errors {
        let s = format!("{}", e);
        assert!(!s.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════
// Keys 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn qualifier_basic() {
    let q = vernal_beans::Qualifier::new("primary").unwrap();
    assert_eq!(q.as_str(), "primary");
}

#[test]
fn scope_key_display() {
    let k = vernal_beans::ScopeKey::of::<String>();
    let s = format!("{}", k);
    assert!(!s.is_empty());
}

#[test]
fn component_key_display() {
    let k = vernal_beans::ComponentKey::of::<String>();
    let s = format!("{}", k);
    assert!(!s.is_empty());
}

#[test]
fn trait_key_display() {
    let k = vernal_beans::TraitKey::of::<dyn std::fmt::Debug>();
    let s = format!("{}", k);
    assert!(!s.is_empty());
}
