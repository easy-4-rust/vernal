//! 边缘情况覆盖测试 — container.rs / root_bean_definition.rs / generic_bean_definition 等。

use std::any::Any;
use std::sync::Arc;
use vernal_beans::{
    Autowire, BeanDefinition, ComponentDefinition, ComponentKey, Container, Qualifier,
    RegistryBuilder, ResolveError, Resolver, Scope,
    autowire_capable_bean_factory::AutowireCapableBeanFactory, bean_factory::BeanFactory,
};

fn make_container() -> Container {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }));
    Container::new(b.build().unwrap())
}

// ── container.rs: resolve_dependency paths ────────────────────────────

#[test]
fn container_resolve_dep_success() {
    use vernal_beans::dependency_descriptor::DependencyDescriptor;
    let c = make_container();
    let desc = DependencyDescriptor::for_field(
        std::any::TypeId::of::<String>(),
        std::any::type_name::<String>(),
    );
    assert!(c.resolve_dependency(&desc, None).unwrap().is_some());
}

#[test]
fn container_resolve_dep_not_found() {
    use vernal_beans::dependency_descriptor::DependencyDescriptor;
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let desc = DependencyDescriptor::for_field(
        std::any::TypeId::of::<String>(),
        std::any::type_name::<String>(),
    );
    assert!(c.resolve_dependency(&desc, None).is_err());
}

#[test]
fn container_resolve_dep_multiple() {
    use vernal_beans::dependency_descriptor::DependencyDescriptor;
    let mut b = RegistryBuilder::new();
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "a".to_string())
            .qualified(Qualifier::new("q1").unwrap()),
    );
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
            .qualified(Qualifier::new("q2").unwrap()),
    );
    let c = Container::new(b.build().unwrap());
    let desc = DependencyDescriptor::for_field(
        std::any::TypeId::of::<String>(),
        std::any::type_name::<String>(),
    );
    let err = c.resolve_dependency(&desc, None).unwrap_err();
    assert!(format!("{}", err).contains("Multiple"));
}

// ── container.rs: get_bean_by_key ─────────────────────────────────────

#[test]
fn container_get_bean_by_key_ok() {
    assert!(
        make_container()
            .get_bean_by_key(&ComponentKey::of::<String>())
            .is_ok()
    );
}

#[test]
fn container_get_bean_by_key_missing() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(c.get_bean_by_key(&ComponentKey::of::<String>()).is_err());
}

// ── container.rs: get_bean_by_type_id (lines 722-732) ─────────────────

#[test]
fn container_get_bean_by_type_id_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(
        c.get_bean_by_type_id(std::any::TypeId::of::<String>())
            .is_err()
    );
}

#[test]
fn container_get_bean_by_type_id_ambig() {
    let mut b = RegistryBuilder::new();
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "a".to_string())
            .qualified(Qualifier::new("q1").unwrap()),
    );
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
            .qualified(Qualifier::new("q2").unwrap()),
    );
    let c = Container::new(b.build().unwrap());
    assert!(
        c.get_bean_by_type_id(std::any::TypeId::of::<String>())
            .is_err()
    );
}

// ── container.rs: create_bean not found ───────────────────────────────

#[test]
fn container_create_bean_missing() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(c.create_bean("NonExistent").is_err());
}

// ── container.rs: autowire_bean ───────────────────────────────────────

#[test]
fn container_autowire_bean_multiple() {
    let mut b = RegistryBuilder::new();
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "a".to_string())
            .qualified(Qualifier::new("q1").unwrap()),
    );
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
            .qualified(Qualifier::new("q2").unwrap()),
    );
    let c = Container::new(b.build().unwrap());
    assert!(
        c.autowire_bean(Arc::new("x".to_string()) as Arc<dyn Any + Send + Sync>)
            .is_ok()
    );
}

#[test]
fn container_autowire_bean_no_match() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(
        c.autowire_bean(Arc::new(42i32) as Arc<dyn Any + Send + Sync>)
            .is_ok()
    );
}

#[test]
fn container_autowire_bean_with_deps() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }));
    b.register(
        ComponentDefinition::singleton::<i32, _>(|r: &Resolver| {
            let _ = r.resolve::<String>().unwrap();
            42i32
        })
        .depends_on::<String>(),
    );
    let c = Container::new(b.build().unwrap());
    assert!(
        c.autowire_bean(Arc::new("t".to_string()) as Arc<dyn Any + Send + Sync>)
            .is_ok()
    );
}

// ── container.rs: initialize_bean with failing PostProcessor ──────────

use vernal_beans::bean_post_processor::BeanPostProcessor;

struct FailPP;
impl BeanPostProcessor for FailPP {
    fn post_process_before_initialization(
        &self,
        _b: Arc<dyn Any + Send + Sync>,
        _n: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Err("before error".into())
    }
    fn post_process_after_initialization(
        &self,
        _b: Arc<dyn Any + Send + Sync>,
        _n: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Err("after error".into())
    }
}

#[test]
fn container_init_bean_failing_pp() {
    let mut c = make_container();
    c.add_bean_post_processor(Arc::new(FailPP));
    assert!(
        c.initialize_bean(Arc::new("x".to_string()) as Arc<dyn Any + Send + Sync>, "t")
            .is_ok()
    );
}

// ── container.rs: configure_bean with PostProcessor ──────────────────

struct NoopPP;
impl BeanPostProcessor for NoopPP {
    fn post_process_before_initialization(
        &self,
        b: Arc<dyn Any + Send + Sync>,
        _n: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }
    fn post_process_after_initialization(
        &self,
        b: Arc<dyn Any + Send + Sync>,
        _n: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }
}

#[test]
fn container_configure_bean_with_pp() {
    let mut c = make_container();
    c.add_bean_post_processor(Arc::new(NoopPP));
    assert!(
        c.configure_bean(Arc::new("x".to_string()) as Arc<dyn Any + Send + Sync>, "t")
            .is_ok()
    );
}

// ── container.rs: resolve_named_bean ─────────────────────────────────

#[test]
fn container_resolve_named_bean_ok() {
    let r = make_container().resolve_named_bean(std::any::TypeId::of::<String>());
    assert!(r.is_ok());
}

#[test]
fn container_resolve_named_bean_missing() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(
        c.resolve_named_bean(std::any::TypeId::of::<String>())
            .is_err()
    );
}

// ── container.rs: autowire mode variations ───────────────────────────

#[test]
fn container_autowire_modes() {
    let c = make_container();
    assert!(
        c.autowire(std::any::type_name::<String>(), 0, false)
            .is_ok()
    );
    assert!(
        c.autowire(std::any::type_name::<String>(), 1, false)
            .is_ok()
    );
    assert!(
        c.autowire(std::any::type_name::<String>(), 2, false)
            .is_ok()
    );
    assert!(
        c.autowire(std::any::type_name::<String>(), 3, false)
            .is_ok()
    );
    assert!(
        c.autowire(std::any::type_name::<String>(), 99, false)
            .is_err()
    );
}

#[test]
fn container_autowire_prop_modes() {
    let c = make_container();
    let b = Arc::new(42i32) as Arc<dyn Any + Send + Sync>;
    assert!(c.autowire_bean_properties(b.clone(), 0, false).is_ok());
    assert!(c.autowire_bean_properties(b.clone(), 1, false).is_ok());
    assert!(c.autowire_bean_properties(b.clone(), 2, false).is_ok());
    assert!(c.autowire_bean_properties(b, 99, false).is_ok());
}

// ── container.rs: is_singleton / is_prototype / get_type / is_type_match / get_aliases ─

#[test]
fn container_is_singleton() {
    let c = make_container();
    assert!(c.is_singleton(&ComponentKey::of::<String>()).unwrap());
    assert!(!c.is_prototype(&ComponentKey::of::<String>()).unwrap());
    assert!(c.is_singleton(&ComponentKey::of::<String>()).is_ok());
}

#[test]
fn container_is_singleton_missing() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(c.is_singleton(&ComponentKey::of::<String>()).is_err());
}

#[test]
fn container_get_type_ok() {
    assert_eq!(
        make_container()
            .get_type(&ComponentKey::of::<String>())
            .unwrap(),
        Some(std::any::type_name::<String>())
    );
}

#[test]
fn container_get_type_missing() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(c.get_type(&ComponentKey::of::<String>()).is_err());
}

#[test]
fn container_is_type_match_ok() {
    let c = make_container();
    assert!(c.is_type_match(
        &ComponentKey::of::<String>(),
        std::any::TypeId::of::<String>()
    ));
    assert!(!c.is_type_match(&ComponentKey::of::<String>(), std::any::TypeId::of::<i32>()));
}

#[test]
fn container_get_aliases() {
    assert!(
        make_container()
            .get_aliases(&ComponentKey::of::<String>())
            .is_empty()
    );
}

// ── container.rs: contains_bean ──────────────────────────────────────

#[test]
fn container_contains() {
    assert!(make_container().contains_bean(&ComponentKey::of::<String>()));
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(!c.contains_bean(&ComponentKey::of::<String>()));
}

// ── container.rs: get_bean_provider ──────────────────────────────────

#[test]
fn container_get_bean_provider_ok() {
    let c = make_container();
    assert!(
        c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>())
            .is_ok()
    );
}

#[test]
fn container_object_provider_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let p = c
        .get_bean_provider_by_type_id(std::any::TypeId::of::<String>())
        .unwrap();
    assert!(p.get().is_err());
    assert!(p.if_available().is_none());
    let _ = p.stream();
    let _ = p.get_if_unique();
}

#[test]
fn container_object_provider_with_bean() {
    let c = make_container();
    let p = c
        .get_bean_provider_by_type_id(std::any::TypeId::of::<String>())
        .unwrap();
    let _ = p.stream();
    // trigger singleton resolution
    let _ = p.get();
}

// ── container.rs: lifecycle methods ───────────────────────────────────

#[test]
fn container_apply_property_values() {
    let c = make_container();
    assert!(
        c.apply_bean_property_values(Arc::new("x".to_string()) as Arc<dyn Any + Send + Sync>, "t")
            .is_ok()
    );
}

#[test]
fn container_destroy_bean() {
    assert!(make_container().destroy_bean_instance("t", &"x").is_ok());
}

// ── container.rs: type_converter ─────────────────────────────────────

#[test]
fn container_type_converter_none() {
    let mut c = make_container();
    c.set_type_converter(None);
    assert!(c.type_converter().is_none());
}

// ── container.rs: BeanDefinitionRegistry ─────────────────────────────

#[test]
fn container_register_dup() {
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    c.register_bean_definition("dup".into(), Box::new(RootBeanDefinition::new()))
        .unwrap();
    let r = c.register_bean_definition("dup".into(), Box::new(RootBeanDefinition::new()));
    assert!(r.is_err());
}

#[test]
fn container_get_bean_def_none() {
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(c.get_bean_definition("Nope").is_none());
}

// ── RootBeanDefinition ───────────────────────────────────────────────

#[test]
fn root_bean_def_all_setters_getters() {
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    let mut d = RootBeanDefinition::new();
    d.set_bean_class_name("com.example.T");
    d.set_parent_name("p");
    d.set_scope(Scope::Transient);
    d.set_lazy_init(true);
    d.set_abstract(true);
    d.set_autowire_candidate(false);
    d.set_primary(true);
    d.set_fallback(true);
    d.set_synthetic(true);
    d.set_role(2);
    d.set_description("desc");
    d.add_depends_on("d1");
    d.add_depends_on("d2");
    d.set_autowire_mode(Autowire::ByType);
    d.set_init_method_name("init");
    d.set_destroy_method_name("destroy");
    d.set_factory_bean_name("fb");
    d.set_factory_method_name("fm");
    d.set_init_order(42);

    assert_eq!(d.bean_class_name(), "com.example.T");
    assert_eq!(d.parent_name(), Some("p"));
    assert_eq!(d.scope(), Scope::Transient);
    assert!(d.is_lazy_init());
    assert!(d.is_abstract());
    assert!(!d.is_autowire_candidate());
    assert!(d.is_primary());
    assert!(d.is_fallback());
    assert!(d.is_synthetic());
    assert_eq!(d.role(), 2);
    assert_eq!(d.description(), Some("desc"));
    assert_eq!(d.depends_on(), &["d1", "d2"]);
    assert_eq!(d.autowire_mode(), Autowire::ByType);
    assert_eq!(d.init_method_name(), Some("init"));
    assert_eq!(d.destroy_method_name(), Some("destroy"));
    assert_eq!(d.factory_bean_name(), Some("fb"));
    assert_eq!(d.factory_method_name(), Some("fm"));
    assert_eq!(d.init_order_value(), 42);
}

#[test]
fn root_bean_def_bean_def_trait() {
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    let mut d = RootBeanDefinition::new();
    d.set_bean_class_name("T");
    d.set_scope(Scope::Singleton);
    d.set_lazy_init(true);
    d.set_primary(true);
    d.set_fallback(false);
    d.set_autowire_candidate(false);
    d.set_role(1);
    d.set_description("d");
    d.set_parent_name("p");
    d.set_factory_bean_name("fb");
    d.set_factory_method_name("fm");
    d.set_init_method_name("init");
    d.set_destroy_method_name("destroy");
    d.set_abstract(true);

    use vernal_beans::bean_definition::BeanDefinition;
    assert_eq!(d.bean_class_name(), "T");
    assert_eq!(d.scope(), Scope::Singleton);
    assert!(d.is_lazy_init());
    assert!(d.is_primary());
    assert!(!d.is_fallback());
    assert!(!d.is_autowire_candidate());
    assert_eq!(d.role(), 1);
    assert_eq!(d.description(), Some("d"));
    assert_eq!(d.parent_name(), Some("p"));
    assert_eq!(d.factory_bean_name(), Some("fb"));
    assert_eq!(d.factory_method_name(), Some("fm"));
    assert_eq!(d.init_method_name(), Some("init"));
    assert_eq!(d.destroy_method_name(), Some("destroy"));
    assert!(d.is_abstract());
}

#[test]
fn root_bean_def_default_trait() {
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    let d = RootBeanDefinition::default();
    assert_eq!(d.bean_class_name(), "unknown");
    assert_eq!(d.scope(), Scope::Singleton);
    assert!(!d.is_lazy_init());
    assert!(!d.is_primary());
    assert!(!d.is_fallback());
    assert!(d.is_autowire_candidate());
    assert!(!d.is_abstract());
    assert!(!d.is_synthetic());
    assert_eq!(d.role(), 0);
    assert!(d.description().is_none());
    assert!(d.parent_name().is_none());
    assert!(d.factory_bean_name().is_none());
    assert!(d.factory_method_name().is_none());
    assert!(d.init_method_name().is_none());
    assert!(d.destroy_method_name().is_none());
}

#[test]
fn root_bean_def_cav_pv() {
    use vernal_beans::constructor_argument_values::{ConstructorArgumentValues, ValueHolder};
    use vernal_beans::root_bean_definition::RootBeanDefinition;

    let mut d = RootBeanDefinition::new();
    d.get_constructor_argument_values_mut()
        .add_generic_argument_value(ValueHolder::new(
            Arc::new(42i32) as Arc<dyn Any + Send + Sync>
        ));
    assert_eq!(d.constructor_argument_values().argument_count(), 1);

    d.get_property_values_mut()
        .add(vernal_beans::property_value::PropertyValue::new(
            "n",
            Arc::new("v".to_string()) as Arc<dyn Any + Send + Sync>,
        ));
    assert_eq!(d.property_values().get_property_values().len(), 1);
}

// ── GenericBeanDefinition ──────────────────────────────────────────

#[test]
fn generic_bean_def_setters_getters() {
    use vernal_beans::generic_bean_definition::GenericBeanDefinition;
    let mut d = GenericBeanDefinition::new();
    d.set_bean_class_name("com.example.G");
    d.set_scope(Scope::Transient);
    d.set_lazy_init(true);
    d.set_abstract(true);
    d.set_autowire_candidate(false);
    d.set_primary(true);
    d.set_fallback(true);
    d.set_synthetic(true);
    d.set_role(2);
    d.set_description("gen");
    d.set_autowire_mode(Autowire::ByType);
    d.set_init_method_name("init");
    d.set_destroy_method_name("destroy");
    d.set_factory_bean_name("fb");
    d.set_factory_method_name("fm");

    assert_eq!(d.get_bean_class_name(), Some("com.example.G"));
    assert_eq!(d.scope(), Scope::Transient);
    assert!(d.is_lazy_init());
    assert!(d.is_abstract());
    assert!(!d.is_autowire_candidate());
    assert!(d.is_primary());
    assert!(d.is_fallback());
    assert!(d.is_synthetic());
    assert_eq!(d.role(), 2);
    assert_eq!(d.description(), Some("gen"));
    assert_eq!(d.autowire_mode(), Autowire::ByType);
    assert_eq!(d.init_method_name(), Some("init"));
    assert_eq!(d.destroy_method_name(), Some("destroy"));
    assert_eq!(d.factory_bean_name(), Some("fb"));
    assert_eq!(d.factory_method_name(), Some("fm"));
}

// ── ResolveError display ────────────────────────────────────────────

#[test]
fn resolve_error_displays() {
    let _ = format!(
        "{}",
        ResolveError::TypeMismatch {
            component: ComponentKey::of::<String>()
        }
    );
    let _ = format!(
        "{}",
        ResolveError::CircularRuntime {
            path: vec!["A".to_string()]
        }
    );
    let _ = format!(
        "{}",
        ResolveError::ScopeNotActive {
            component: ComponentKey::of::<String>(),
            scope: vernal_beans::ScopeKey::of::<String>(),
        }
    );
    let _ = format!(
        "{}",
        ResolveError::Ambiguous {
            component: "t".to_string(),
            candidates: vec!["A".to_string()],
            path: vec!["r".to_string()],
        }
    );
    let _ = format!(
        "{}",
        ResolveError::NotFound {
            component: "t".to_string(),
            path: vec!["r".to_string()]
        }
    );
    let _ = format!(
        "{}",
        ResolveError::UndeclaredDependency {
            component: ComponentKey::of::<String>(),
            dependency: "d".to_string(),
        }
    );
    let _ = format!(
        "{}",
        ResolveError::TraitBindingTypeMismatch {
            binding: vernal_beans::TraitKey::of::<dyn std::fmt::Debug + Send + Sync>(),
            target: vernal_beans::ComponentKey::of::<i32>(),
        }
    );
}

// ── Autowire enum ────────────────────────────────────────────────────

#[test]
fn autowire_enum_int_values() {
    assert_eq!(Autowire::No as i32, 0);
    assert_eq!(Autowire::ByName as i32, 1);
    assert_eq!(Autowire::ByType as i32, 2);
    assert_eq!(Autowire::Constructor as i32, 3);
}

// ── BeanFactoryUtils ────────────────────────────────────────────────

#[test]
fn bean_factory_utils_basics() {
    use vernal_beans::bean_factory_utils::BeanFactoryUtils;
    assert_eq!(BeanFactoryUtils::transformed_bean_name("&x"), "x");
    assert_eq!(BeanFactoryUtils::transformed_bean_name("x"), "x");
    assert!(BeanFactoryUtils::is_factory_bean("&x"));
    assert!(!BeanFactoryUtils::is_factory_bean("x"));
    assert!(!BeanFactoryUtils::is_factory_bean(""));
}

// ── Qualifier ────────────────────────────────────────────────────────

#[test]
fn qualifier_methods() {
    let q = Qualifier::new("test").unwrap();
    assert_eq!(q.as_str(), "test");
    assert_eq!(format!("{}", q), "test");
    assert!(Qualifier::new("").is_err());
}

// ── MutablePropertyValues / PropertyValue / ConstructorArgumentValues ─

#[test]
fn mpv_operations() {
    use vernal_beans::mutable_property_values::MutablePropertyValues;
    use vernal_beans::property_value::PropertyValue;

    let mut mpv = MutablePropertyValues::new();
    assert!(mpv.is_empty());
    mpv.add(PropertyValue::new(
        "n",
        Arc::new("v".to_string()) as Arc<dyn Any + Send + Sync>,
    ));
    assert!(!mpv.is_empty());
    assert!(mpv.contains("n"));
    assert!(mpv.get("n").is_some());
    assert_eq!(mpv.len(), 1);
    mpv.clear();
    assert!(mpv.is_empty());
}

#[test]
fn mpv_add_value() {
    use vernal_beans::mutable_property_values::MutablePropertyValues;
    let mut mpv = MutablePropertyValues::new();
    mpv.add_value("n", Arc::new("v".to_string()));
    assert!(!mpv.is_empty());
}

#[test]
fn property_value_name_value() {
    use vernal_beans::property_value::PropertyValue;
    let pv = PropertyValue::new("c", Arc::new(42i32) as Arc<dyn Any + Send + Sync>);
    assert_eq!(pv.name(), "c");
    let _v: &Arc<dyn Any + Send + Sync> = pv.value();
}

#[test]
fn constructor_argument_values_ops() {
    use vernal_beans::constructor_argument_values::{ConstructorArgumentValues, ValueHolder};
    let mut cav = ConstructorArgumentValues::new();
    assert!(cav.is_empty());
    assert_eq!(cav.argument_count(), 0);
    cav.add_generic_argument_value(ValueHolder::new(
        Arc::new("t".to_string()) as Arc<dyn Any + Send + Sync>
    ));
    assert_eq!(cav.argument_count(), 1);
    cav.add_indexed_argument_value(
        0,
        ValueHolder::new(Arc::new(42i32) as Arc<dyn Any + Send + Sync>),
    );
    assert_eq!(cav.argument_count(), 2);
    assert!(cav.has_indexed_argument_value(0));
    assert!(cav.get_indexed_argument_value(0).is_some());
    assert!(cav.get_generic_argument_value("t").is_none());
    let _ = cav.indexed_argument_values();
    let _ = cav.generic_argument_values();
    cav.clear();
    assert!(cav.is_empty());
}

// ── DependencyDescriptor ────────────────────────────────────────────

#[test]
fn dependency_descriptor_basics() {
    use vernal_beans::dependency_descriptor::DependencyDescriptor;
    let d = DependencyDescriptor::for_field(std::any::TypeId::of::<String>(), "S");
    assert_eq!(d.type_id(), std::any::TypeId::of::<String>());
    assert_eq!(d.type_name(), "S");
    assert!(d.qualifier().is_none());
}

#[test]
fn dependency_descriptor_with_q() {
    use vernal_beans::dependency_descriptor::DependencyDescriptor;
    let q = Qualifier::new("p").unwrap();
    let d = DependencyDescriptor::for_field(std::any::TypeId::of::<String>(), "S")
        .with_qualifier("primary");
    assert!(d.qualifier().is_some());

    let d2 = DependencyDescriptor::for_constructor_parameter(0, std::any::TypeId::of::<i32>(), "I");
    assert_eq!(d2.type_name(), "I");
}

// ── StandardBeanExpressionResolver ──────────────────────────────────

#[test]
fn expression_resolver_basic() {
    use vernal_beans::bean_expression_resolver::BeanExpressionResolver;
    use vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver;
    let r = StandardBeanExpressionResolver::new();
    assert!(r.evaluate("myBean", None).unwrap().is_none());
    assert!(r.evaluate("", None).unwrap().is_none());
}

// ── SimpleScope ──────────────────────────────────────────────────────

#[test]
fn scope_key_of() {
    use vernal_beans::ScopeKey;
    let key = ScopeKey::of::<String>();
    assert!(!key.type_name().is_empty());
}

// ── RegistryBuilder ─────────────────────────────────────────────────

#[test]
fn registry_builder_ops() {
    let mut b = RegistryBuilder::new();
    assert!(!b.contains::<String>());
    assert!(b.is_empty());
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "h".to_string()
    }));
    assert!(b.contains::<String>());
    let r = b.build().unwrap();
    assert_eq!(r.len(), 1);
    assert!(!r.is_empty());
}

#[test]
fn registry_remove_not_found() {
    let mut b = RegistryBuilder::new();
    assert!(b.remove_by_key(&ComponentKey::of::<String>()).is_err());
    assert!(b.remove::<String>().is_err());
}

// ── ConfigurationClassPostProcessor ──────────────────────────────────

#[test]
fn config_post_processor_basics() {
    use vernal_beans::configuration_class_post_processor::ConfigurationClassPostProcessor;
    let mut p = ConfigurationClassPostProcessor::new();
    assert_eq!(p.registered_count(), 0);
    assert!(p.registered_configurations().is_empty());
    p.register_configuration("com.example.C");
    assert_eq!(p.registered_count(), 1);
    assert_eq!(p.registered_configurations()[0], "com.example.C");
}
