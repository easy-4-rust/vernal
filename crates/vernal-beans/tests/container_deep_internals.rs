use std::sync::Arc;
use vernal_beans::{
    AutowireCapableBeanFactory, BeanDefinitionRegistry, BeanFactory, ComponentDefinition,
    ComponentKey, Container, Qualifier, RegistryBuilder,
    configurable_listable_bean_factory::ConfigurableListableBeanFactory,
    singleton_bean_registry::SingletonBeanRegistry,
};

fn container() -> Container {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }));
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32));
    Container::new(b.build().unwrap())
}

// ── ContainerObjectProvider edge cases ──────────────────────────

#[test]
fn provider_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let p = c
        .get_bean_provider_by_type_id(std::any::TypeId::of::<String>())
        .unwrap();
    assert!(p.if_available().is_none());
    assert!(p.get().is_err());
}

#[test]
fn provider_with_bean() {
    let c = container();
    let p = c
        .get_bean_provider_by_type_id(std::any::TypeId::of::<String>())
        .unwrap();
    let _avail = p.if_available();
    let _ = p.get();
    let _avail2 = p.if_available();
    let _unique = p.get_if_unique();
    let _stream = p.stream();
    let _ordered = p.ordered_stream();
}

// ── SingletonBeanRegistry ──────────────────────────────────────

#[test]
fn sbr_register_and_query() {
    let c = container();
    c.register_singleton("s", Arc::new(99i32));
    assert!(c.contains_singleton("s"));
    assert_eq!(c.singleton_count(), 1);
    let names = c.singleton_names();
    assert!(names.contains(&"s".to_string()));
}

#[test]
fn sbr_mutex() {
    let c = container();
    let m1 = c.singleton_mutex();
    let m2 = c.singleton_mutex();
    assert!(Arc::ptr_eq(&m1, &m2));
}

#[test]
fn sbr_via_once_lock() {
    let c = container();
    let _bean = c.get_bean_by_key(&ComponentKey::of::<String>()).unwrap();
    assert!(c.contains_singleton(std::any::type_name::<String>()));
    assert!(c.get_singleton(std::any::type_name::<String>()).is_some());
}

// ── ConfigurableListableBeanFactory ─────────────────────────────

#[test]
fn clbf_is_autowire_candidate_ok() {
    let c = container();
    assert!(c.is_autowire_candidate(std::any::type_name::<String>()));
    assert!(!c.is_autowire_candidate("unknown"));
}

#[test]
fn clbf_freeze_and_instantiate() {
    let mut c = container();
    assert!(!c.is_configuration_frozen());
    c.freeze_configuration();
    assert!(c.is_configuration_frozen());
    assert!(c.pre_instantiate_singletons().is_ok());
}

// ── get_bean_by_type_id paths ──────────────────────────────────

#[test]
fn gbtid_found() {
    let c = container();
    assert!(
        c.get_bean_by_type_id(std::any::TypeId::of::<String>())
            .is_ok()
    );
}

#[test]
fn gbtid_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(
        c.get_bean_by_type_id(std::any::TypeId::of::<String>())
            .is_err()
    );
}

#[test]
fn gbtid_multiple() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "a".to_string()
    }));
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
            .qualified(Qualifier::new("q").unwrap()),
    );
    let c = Container::new(b.build().unwrap());
    assert!(
        c.get_bean_by_type_id(std::any::TypeId::of::<String>())
            .is_err()
    );
}

// ── resolve_named_bean paths ───────────────────────────────────

#[test]
fn rnb_found() {
    let c = container();
    assert!(
        c.resolve_named_bean(std::any::TypeId::of::<String>())
            .is_ok()
    );
}

#[test]
fn rnb_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(
        c.resolve_named_bean(std::any::TypeId::of::<String>())
            .is_err()
    );
}

// ── resolve_dependency paths ───────────────────────────────────

#[test]
fn rd_multiple() {
    use vernal_beans::dependency_descriptor::DependencyDescriptor;
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "a".to_string()
    }));
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
            .qualified(Qualifier::new("q").unwrap()),
    );
    let c = Container::new(b.build().unwrap());
    let desc = DependencyDescriptor::for_field(std::any::TypeId::of::<String>(), "S");
    assert!(c.resolve_dependency(&desc, None).is_err());
}

#[test]
fn rd_optional_not_found() {
    use vernal_beans::dependency_descriptor::DependencyDescriptor;
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let desc = DependencyDescriptor::for_field(std::any::TypeId::of::<String>(), "S");
    assert!(c.resolve_dependency(&desc, None).is_err());
}
