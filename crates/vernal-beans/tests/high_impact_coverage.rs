//! High-impact coverage tests targeting the largest uncovered areas.
use std::any::Any;
use std::sync::Arc;
use vernal_beans::{
    ComponentDefinition, ComponentKey, Container, RegistryBuilder, Qualifier,
    ResolveError, Dependency, TraitKey, Scope,
};

// ── ConstructorArgumentValues ─────────────────────────────────────

#[test]
fn cav_value_holder_basic() {
    use vernal_beans::constructor_argument_values::{ConstructorArgumentValues, ValueHolder};
    let vh = ValueHolder::new(Arc::new(42i32));
    assert!(vh.value().is_some());
    assert!(vh.type_name().is_none());
    assert!(vh.name().is_none());
    assert!(!vh.is_converted());
    assert!(vh.converted_value().is_none());
}

#[test]
fn cav_value_holder_with_type() {
    use vernal_beans::constructor_argument_values::ValueHolder;
    let vh = ValueHolder::with_type(Arc::new("hello".to_string()), "java.lang.String");
    assert!(vh.value().is_some());
    assert_eq!(vh.type_name(), Some("java.lang.String"));
}

#[test]
fn cav_value_holder_with_type_and_name() {
    use vernal_beans::constructor_argument_values::ValueHolder;
    let vh = ValueHolder::with_type_and_name(Arc::new(42i32), "int", "count");
    assert_eq!(vh.type_name(), Some("int"));
    assert_eq!(vh.name(), Some("count"));
}

#[test]
fn cav_value_holder_copy() {
    use vernal_beans::constructor_argument_values::ValueHolder;
    let vh = ValueHolder::with_type(Arc::new(42i32), "int");
    let vh2 = vh.copy();
    assert!(vh2.value().is_some());
    assert_eq!(vh2.type_name(), Some("int"));
}

#[test]
fn cav_value_holder_set_converted() {
    use vernal_beans::constructor_argument_values::ValueHolder;
    let mut vh = ValueHolder::new(Arc::new("42"));
    vh.set_converted_value(Arc::new(42i32));
    assert!(vh.is_converted());
    assert!(vh.converted_value().is_some());
}

#[test]
fn cav_indexed_args() {
    use vernal_beans::constructor_argument_values::{ConstructorArgumentValues, ValueHolder};
    let mut cav = ConstructorArgumentValues::new();
    assert!(cav.indexed_argument_values().is_empty());
    cav.add_indexed_argument_value(0, ValueHolder::new(Arc::new("first")));
    cav.add_indexed_argument_value(1, ValueHolder::new(Arc::new("second")));
    assert!(cav.has_indexed_argument_value(0));
    assert!(!cav.has_indexed_argument_value(99));
    let vh = cav.get_indexed_argument_value(0);
    assert!(vh.is_some());
    assert!(cav.get_indexed_argument_value(99).is_none());
}

#[test]
fn cav_generic_args() {
    use vernal_beans::constructor_argument_values::{ConstructorArgumentValues, ValueHolder};
    let mut cav = ConstructorArgumentValues::new();
    assert!(cav.generic_argument_values().is_empty());
    cav.add_generic_argument_value(ValueHolder::with_type(Arc::new("val"), "String"));
    assert_eq!(cav.generic_argument_values().len(), 1);
    assert!(cav.get_generic_argument_value("String").is_some());
    assert!(cav.get_generic_argument_value("Integer").is_none());
}

#[test]
fn cav_get_argument_value() {
    use vernal_beans::constructor_argument_values::{ConstructorArgumentValues, ValueHolder};
    let mut cav = ConstructorArgumentValues::new();
    cav.add_indexed_argument_value(0, ValueHolder::new(Arc::new("indexed")));
    cav.add_generic_argument_value(ValueHolder::with_type(Arc::new("generic"), "String"));
    let vh = cav.get_argument_value(0, None, None);
    assert!(vh.is_some());
}

#[test]
fn cav_from_other() {
    use vernal_beans::constructor_argument_values::{ConstructorArgumentValues, ValueHolder};
    let mut original = ConstructorArgumentValues::new();
    original.add_indexed_argument_value(0, ValueHolder::new(Arc::new("val")));
    original.add_generic_argument_value(ValueHolder::with_type(Arc::new("gen"), "String"));
    let copy = ConstructorArgumentValues::from_other(&original);
    assert!(copy.has_indexed_argument_value(0));
    assert_eq!(copy.generic_argument_values().len(), 1);
}

#[test]
fn cav_arg_count() {
    use vernal_beans::constructor_argument_values::{ConstructorArgumentValues, ValueHolder};
    let mut cav = ConstructorArgumentValues::new();
    assert_eq!(cav.argument_count(), 0);
    assert!(cav.is_empty());
    cav.add_indexed_argument_value(0, ValueHolder::new(Arc::new("val")));
    assert_eq!(cav.argument_count(), 1);
    assert!(!cav.is_empty());
}

#[test]
fn cav_contains_named() {
    use vernal_beans::constructor_argument_values::{ConstructorArgumentValues, ValueHolder};
    let mut cav = ConstructorArgumentValues::new();
    assert!(!cav.contains_named_argument());
    cav.add_generic_argument_value(ValueHolder::with_type_and_name(Arc::new("val"), "String", "name"));
    assert!(cav.contains_named_argument());
}

#[test]
fn cav_clear() {
    use vernal_beans::constructor_argument_values::{ConstructorArgumentValues, ValueHolder};
    let mut cav = ConstructorArgumentValues::new();
    cav.add_indexed_argument_value(0, ValueHolder::new(Arc::new("val")));
    cav.add_generic_argument_value(ValueHolder::with_type(Arc::new("gen"), "String"));
    assert!(!cav.is_empty());
    cav.clear();
    assert!(cav.is_empty());
}

#[test]
fn cav_add_argument_values() {
    use vernal_beans::constructor_argument_values::{ConstructorArgumentValues, ValueHolder};
    let mut cav1 = ConstructorArgumentValues::new();
    cav1.add_indexed_argument_value(0, ValueHolder::new(Arc::new("val1")));
    let mut cav2 = ConstructorArgumentValues::new();
    cav2.add_indexed_argument_value(1, ValueHolder::new(Arc::new("val2")));
    cav2.add_generic_argument_value(ValueHolder::with_type(Arc::new("gen"), "String"));
    cav1.add_argument_values(&cav2);
    assert!(cav1.has_indexed_argument_value(0));
    assert!(cav1.has_indexed_argument_value(1));
    assert_eq!(cav1.generic_argument_values().len(), 1);
}

// ── InjectionPoint ────────────────────────────────────────────────

#[test]
fn injection_point_basic() {
    use vernal_beans::injection_point::InjectionPoint;
    let ip = InjectionPoint::new(std::any::TypeId::of::<String>(), "alloc::string::String");
    assert_eq!(ip.type_id(), std::any::TypeId::of::<String>());
    assert_eq!(ip.type_name(), "alloc::string::String");
    assert!(ip.containing_bean_name().is_none());
    assert!(ip.member_name().is_none());
    assert!(ip.qualifier().is_none());
}

#[test]
fn injection_point_with_options() {
    use vernal_beans::injection_point::InjectionPoint;
    let ip = InjectionPoint::new(std::any::TypeId::of::<i32>(), "i32")
        .with_containing_bean_name("myBean")
        .with_member_name("count")
        .with_qualifier("primary");
    assert_eq!(ip.containing_bean_name(), Some("myBean"));
    assert_eq!(ip.member_name(), Some("count"));
    assert_eq!(ip.qualifier(), Some("primary"));
}

// ── BeanDefinitionBuilder ─────────────────────────────────────────

#[test]
fn bean_definition_builder_generic() {
    use vernal_beans::bean_definition_builder::BeanDefinitionBuilder;
    use vernal_beans::bean_definition::BeanDefinition;
    let def = BeanDefinitionBuilder::generic("com.example.Service")
        .set_parent_name("parentService")
        .set_scope(Scope::Transient)
        .set_lazy_init(true)
        .set_abstract(true)
        .set_autowire_candidate(false)
        .set_primary(true)
        .set_fallback(true)
        .set_role(1)
        .set_description("A test service")
        .add_depends_on("dataSource")
        .set_init_method("init")
        .set_destroy_method("destroy")
        .build();
    assert!(def.is_lazy_init());
}

#[test]
fn bean_definition_builder_root() {
    use vernal_beans::bean_definition_builder::BeanDefinitionBuilder;
    use vernal_beans::bean_definition::BeanDefinition;
    let def = BeanDefinitionBuilder::root("com.example.RootService")
        .set_scope(Scope::Transient)
        .set_lazy_init(true)
        .set_primary(true)
        .set_init_method("init")
        .set_destroy_method("destroy")
        .add_depends_on("dep1")
        .build();
    assert!(def.is_lazy_init());
    assert!(def.is_primary());
}

#[test]
fn bean_definition_builder_with_constructor_args() {
    use vernal_beans::bean_definition_builder::BeanDefinitionBuilder;
    let def = BeanDefinitionBuilder::generic("com.example.Service")
        .add_constructor_arg_value(Arc::new("arg1"))
        .add_constructor_arg_typed(Arc::new(42i32), "int")
        .add_property_value("name", Arc::new("test"))
        .build();
    assert_eq!(def.scope(), Scope::Singleton);
}

#[test]
fn root_bean_definition_builder_with_methods() {
    use vernal_beans::bean_definition_builder::BeanDefinitionBuilder;
    use vernal_beans::bean_definition::BeanDefinition;
    let def = BeanDefinitionBuilder::root("com.example.Root")
        
        
        
        
        
        
        
        .build();
    assert_eq!(def.bean_class_name(), "com.example.Root");
}

#[test]
fn root_bean_definition_builder_factory() {
    use vernal_beans::bean_definition_builder::BeanDefinitionBuilder;
    use vernal_beans::bean_definition::BeanDefinition;
    let def = BeanDefinitionBuilder::root("com.example.Factory")
        
        
        
        .build();
    assert_eq!(def.bean_class_name(), "com.example.Factory");
}

// ── StaticListableBeanFactory ─────────────────────────────────────

#[test]
fn static_listable_bean_factory_full() {
    use vernal_beans::static_listable_bean_factory::StaticListableBeanFactory;
    let mut bf = StaticListableBeanFactory::new();
    bf.register_singleton("bean1", Arc::new(42i32));
    bf.register_singleton("bean2", Arc::new("hello".to_string()));
    assert!(bf.contains_bean_name("bean1"));
    assert!(!bf.contains_bean_name("bean3"));
    let names = bf.bean_names();
    assert_eq!(names.len(), 2);
}

// ── Container deep coverage ───────────────────────────────────────

#[test]
fn container_resolve_err_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let result: Result<Arc<String>, _> = c.resolve();
    assert!(result.is_err());
}

#[test]
fn container_resolve_qualified_err() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
    let c = Container::new(b.build().unwrap());
    let q = Qualifier::new("nonexistent").unwrap();
    let result: Result<Arc<String>, _> = c.resolve_qualified(&q);
    assert!(result.is_err());
}

#[test]
fn container_resolve_in_scope_err() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let scope = c.open_scope::<String>();
    let result: Result<Arc<String>, _> = c.resolve_in(&scope);
    assert!(result.is_err());
}

#[test]
fn container_resolve_trait_err() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let result: Result<Arc<dyn std::fmt::Debug + Send + Sync>, _> = c.resolve_trait();
    assert!(result.is_err());
}

#[test]
fn container_resolve_all_traits_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let result: Result<Vec<Arc<dyn std::fmt::Debug + Send + Sync>>, _> = c.resolve_all_traits();
    assert!(result.unwrap().is_empty());
}

#[test]
fn container_warm_up_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(c.warm_up().is_ok());
    assert!(c.unused_definitions().is_empty());
}

#[test]
fn container_warm_up_with_beans() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32));
    let c = Container::new(b.build().unwrap());
    assert!(c.warm_up().is_ok());
    assert!(c.unused_definitions().is_empty());
}

#[test]
fn container_post_processor() {
    use vernal_beans::bean_post_processor::BeanPostProcessor;
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    struct PP;
    impl BeanPostProcessor for PP {}
    c.add_bean_post_processor(Arc::new(PP));
    c.add_bean_post_processor(Arc::new(PP));
    assert_eq!(c.bean_post_processor_count(), 2);
}

#[test]
fn container_scope_open() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let scope1 = c.open_scope::<String>();
    let scope2 = c.open_scope::<i32>();
    let scope3 = c.open_scope::<f64>();
    let _ = (scope1, scope2, scope3);
}

#[test]
fn container_transient_tracker() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let tracker = c.transient_tracker();
    let _ = tracker;
}

// ── RegistryBuilder coverage ──────────────────────────────────────

#[test]
fn registry_builder_empty() {
    let b = RegistryBuilder::new();
    let reg = b.build().unwrap();
    let c = Container::new(reg);
    let result: Result<Arc<String>, _> = c.resolve();
    assert!(result.is_err());
}

#[test]

#[test]
fn registry_builder_with_qualifier() {
    let mut b = RegistryBuilder::new();
    let q = Qualifier::new("primary").unwrap();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "primary".to_string()).qualified(q.clone()));
    b.register(ComponentDefinition::singleton::<String, _>(|_| "default".to_string()));
    let c = Container::new(b.build().unwrap());
    let val: Arc<String> = c.resolve_qualified(&q).unwrap();
    assert_eq!(*val, "primary");
}

// ── BeanDefinition trait coverage ─────────────────────────────────

#[test]
fn bean_definition_setters_getters() {
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    use vernal_beans::bean_definition::BeanDefinition;
    let mut rbd = RootBeanDefinition::new();
    rbd.set_bean_class_name("com.example.MyService");
    assert_eq!(rbd.bean_class_name(), "com.example.MyService");
    rbd.set_scope(Scope::Transient);
    assert_eq!(rbd.scope(), Scope::Transient);
    rbd.set_lazy_init(true);
    assert!(rbd.is_lazy_init());
    rbd.set_abstract(true);
    assert!(rbd.is_abstract());
    rbd.set_primary(true);
    assert!(rbd.is_primary());
}

#[test]
fn bean_definition_scope_checks() {
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    use vernal_beans::bean_definition::BeanDefinition;
    let mut rbd = RootBeanDefinition::new();
    assert!(rbd.is_singleton());
    assert!(!rbd.is_prototype());
    rbd.set_scope(Scope::Transient);
    assert!(!rbd.is_singleton());
    assert!(rbd.is_prototype());
}

// ── ScopeContext coverage ─────────────────────────────────────────

#[test]
fn scope_context_key() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let scope = c.open_scope::<String>();
    let _key = scope.key();
}

#[test]
fn scope_context_state() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let scope = c.open_scope::<String>();
    let _state = scope.state();
}

// ── Dependency coverage ───────────────────────────────────────────

#[test]
fn dependency_display() {
    let d1 = Dependency::of::<String>();
    let d2 = Dependency::qualified::<String>(Qualifier::new("q").unwrap());
    let d3 = Dependency::optional_of::<String>();
    let d4 = Dependency::provider_of::<String>();
    let d5 = Dependency::trait_of::<dyn std::fmt::Debug + Send + Sync>();
    assert!(!format!("{}", d1).is_empty());
    assert!(!format!("{}", d2).is_empty());
    assert!(!format!("{}", d3).is_empty());
    assert!(!format!("{}", d4).is_empty());
    assert!(!format!("{}", d5).is_empty());
}

// ── ResolveError coverage ─────────────────────────────────────────

#[test]
fn resolve_error_display_all() {
    let errors: Vec<ResolveError> = vec![
        ResolveError::NotFound { component: "t".into(), path: vec!["r".into()] },
        ResolveError::Ambiguous { component: "t".into(), candidates: vec!["a".into(), "b".into()], path: vec!["r".into()] },
        ResolveError::UndeclaredDependency { component: ComponentKey::of::<String>(), dependency: "d".into() },
        ResolveError::TypeMismatch { component: ComponentKey::of::<String>() },
        ResolveError::TraitBindingTypeMismatch { binding: TraitKey::of::<dyn std::fmt::Debug>(), target: ComponentKey::of::<i32>() },
        ResolveError::Construction { component: ComponentKey::of::<String>(), source: Arc::new(std::io::Error::new(std::io::ErrorKind::Other, "e")) },
        ResolveError::CircularRuntime { path: vec!["a".into(), "b".into(), "c".into()] },
        ResolveError::ProviderUsedDuringConstruction { component: ComponentKey::of::<String>(), dependency: "d".into() },
        ResolveError::ScopeNotActive { component: ComponentKey::of::<String>(), scope: vernal_beans::ScopeKey::of::<String>() },
        ResolveError::ScopeOwnerMismatch { scope: vernal_beans::ScopeKey::of::<String>() },
    ];
    for e in &errors {
        let s = format!("{}", e);
        assert!(!s.is_empty());
    }
}

// ── ComponentKey/TraitKey/ScopeKey display ────────────────────────

#[test]
fn keys_display() {
    let ck = ComponentKey::of::<String>();
    let tk = TraitKey::of::<dyn std::fmt::Debug>();
    let sk = vernal_beans::ScopeKey::of::<String>();
    assert!(!format!("{}", ck).is_empty());
    assert!(!format!("{}", tk).is_empty());
    assert!(!format!("{}", sk).is_empty());
}
