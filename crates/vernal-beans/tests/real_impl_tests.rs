/// 测试真实实现的核心文件。
use std::any::Any;
use std::sync::Arc;

// ═══════════════════════════════════════════════════════════════════
// BeanWrapperImpl 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bean_wrapper_new() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    let _ = wrapper.property_count();
}

#[test]
fn bean_wrapper_register_property() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    wrapper.register_property("name", std::any::TypeId::of::<String>());
    assert!(wrapper.is_readable("name"));
    assert!(wrapper.is_writable("name"));
}

#[test]
fn bean_wrapper_set_get_property() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    wrapper.register_property("name", std::any::TypeId::of::<String>());
    wrapper.set_property_value("name", Arc::new("Alice".to_string())).unwrap();
    let val = wrapper.get_property_value("name").unwrap();
    assert_eq!(val.downcast_ref::<String>().unwrap(), "Alice");
}

#[test]
fn bean_wrapper_readonly() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    wrapper.register_readonly_property("ro", std::any::TypeId::of::<i32>());
    assert!(wrapper.is_readable("ro"));
    assert!(!wrapper.is_writable("ro"));
    assert!(wrapper.set_property_value("ro", Arc::new(42i32)).is_err());
}

#[test]
fn bean_wrapper_nested_property() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    use std::collections::HashMap;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    wrapper.register_property("address", std::any::TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>());
    let mut inner = HashMap::new();
    inner.insert("city".to_string(), Arc::new("Beijing".to_string()) as Arc<dyn Any + Send + Sync>);
    wrapper.set_property_value("address", Arc::new(inner)).unwrap();
    let val = wrapper.get_property_value("address.city");
    assert!(val.is_ok());
    assert_eq!(val.unwrap().downcast_ref::<String>().unwrap(), "Beijing");
}

#[test]
fn bean_wrapper_property_type() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    wrapper.register_property("name", std::any::TypeId::of::<String>());
    assert_eq!(wrapper.get_property_type("name"), Some(std::any::TypeId::of::<String>()));
    assert_eq!(wrapper.get_property_type("missing"), None);
}

#[test]
fn bean_wrapper_property_names() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    wrapper.register_property("a", std::any::TypeId::of::<i32>());
    wrapper.register_property("b", std::any::TypeId::of::<String>());
    let mut names = wrapper.get_property_names();
    names.sort();
    assert_eq!(names, vec!["a", "b"]);
}

#[test]
fn bean_wrapper_nonexistent_property() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    assert!(wrapper.get_property_value("missing").is_err());
}

#[test]
fn bean_wrapper_property_count() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    assert_eq!(wrapper.property_count(), 0);
}

// ═══════════════════════════════════════════════════════════════════
// RootBeanDefinition 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn root_bean_definition_basic() {
    use vernal_beans::RootBeanDefinition;
    use vernal_beans::BeanDefinition;
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
    use vernal_beans::BeanDefinition;
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
    use vernal_beans::BeanDefinition;
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
    let d2 = vernal_beans::Dependency::qualified::<String>(vernal_beans::Qualifier::new("q").unwrap());
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
    use vernal_beans::{ResolveError, ComponentKey, TraitKey, ScopeKey};
    let errors: Vec<ResolveError> = vec![
        ResolveError::NotFound { component: "t".into(), path: vec!["r".into()] },
        ResolveError::Ambiguous { component: "t".into(), candidates: vec!["a".into(), "b".into()], path: vec!["r".into()] },
        ResolveError::UndeclaredDependency { component: ComponentKey::of::<String>(), dependency: "d".into() },
        ResolveError::TypeMismatch { component: ComponentKey::of::<String>() },
        ResolveError::TraitBindingTypeMismatch { binding: TraitKey::of::<dyn std::fmt::Debug>(), target: ComponentKey::of::<i32>() },
        ResolveError::Construction { component: ComponentKey::of::<String>(), source: Arc::new(std::io::Error::new(std::io::ErrorKind::Other, "e")) },
        ResolveError::CircularRuntime { path: vec!["a".into(), "b".into(), "c".into()] },
        ResolveError::ProviderUsedDuringConstruction { component: ComponentKey::of::<String>(), dependency: "d".into() },
        ResolveError::ScopeNotActive { component: ComponentKey::of::<String>(), scope: ScopeKey::of::<String>() },
        ResolveError::ScopeOwnerMismatch { scope: ScopeKey::of::<String>() },
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
