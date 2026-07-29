use std::sync::Arc;
use vernal_beans::{
    AutowireCapableBeanFactory, BeanFactory, ComponentDefinition, ComponentKey,
    ConfigurableBeanFactory, Container, ListableBeanFactory, RegistryBuilder, ResolveError,
    ScopeKey, TraitKey, bean_scope::BeanScope,
    configurable_listable_bean_factory::ConfigurableListableBeanFactory,
};

#[test]
fn clbf_ignore_dep_type() {
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    c.ignore_dependency_type(std::any::TypeId::of::<String>());
    assert!(!c.is_autowire_candidate(std::any::type_name::<String>()));
}

#[test]
fn clbf_ignore_dep_intf() {
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    c.ignore_dependency_interface(std::any::TypeId::of::<String>());
    assert!(!c.is_autowire_candidate(std::any::type_name::<String>()));
}

#[test]
fn clbf_register_resolvable() {
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    c.register_resolvable_dependency(std::any::TypeId::of::<String>(), Arc::new("v".to_string()));
}

#[test]
fn cbf_scope_reg_and_list() {
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    struct S;
    impl BeanScope for S {
        fn get(
            &self,
            _n: &str,
            _f: &dyn Fn() -> Box<dyn std::any::Any + Send + Sync>,
        ) -> Result<Box<dyn std::any::Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(Box::new("v"))
        }
    }
    c.register_scope("s", Box::new(S));
    assert!(c.registered_scope_names().contains(&"s".into()));
    assert!(c.get_registered_scope("s").is_none());
    assert!(c.get_registered_scope("x").is_none());
}

#[test]
fn type_match_positive() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "h".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    assert!(c.is_type_match(
        &ComponentKey::of::<String>(),
        std::any::TypeId::of::<String>()
    ));
    assert!(!c.is_type_match(&ComponentKey::of::<String>(), std::any::TypeId::of::<i32>()));
}

#[test]
fn autowire_all_modes() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "h".to_string()
    }));
    let c = Container::new(b.build().unwrap());
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
fn resolve_error_all_display() {
    let v: Vec<String> = vec![
        format!(
            "{}",
            ResolveError::NotFound {
                component: "t".into(),
                path: vec!["r".into()]
            }
        ),
        format!(
            "{}",
            ResolveError::Ambiguous {
                component: "t".into(),
                candidates: vec!["a".into()],
                path: vec!["r".into()]
            }
        ),
        format!(
            "{}",
            ResolveError::UndeclaredDependency {
                component: ComponentKey::of::<String>(),
                dependency: "d".into()
            }
        ),
        format!(
            "{}",
            ResolveError::TypeMismatch {
                component: ComponentKey::of::<String>()
            }
        ),
        format!(
            "{}",
            ResolveError::TraitBindingTypeMismatch {
                binding: TraitKey::of::<dyn std::fmt::Debug>(),
                target: ComponentKey::of::<i32>()
            }
        ),
        format!(
            "{}",
            ResolveError::Construction {
                component: ComponentKey::of::<String>(),
                source: Arc::new(std::io::Error::new(std::io::ErrorKind::Other, "e"))
            }
        ),
        format!(
            "{}",
            ResolveError::CircularRuntime {
                path: vec!["a".into()]
            }
        ),
        format!(
            "{}",
            ResolveError::ProviderUsedDuringConstruction {
                component: ComponentKey::of::<String>(),
                dependency: "d".into()
            }
        ),
        format!(
            "{}",
            ResolveError::ScopeNotActive {
                component: ComponentKey::of::<String>(),
                scope: ScopeKey::of::<String>()
            }
        ),
        format!(
            "{}",
            ResolveError::ScopeOwnerMismatch {
                scope: ScopeKey::of::<String>()
            }
        ),
    ];
    assert_eq!(v.len(), 10);
    for s in &v {
        assert!(!s.is_empty());
    }
}
