use std::sync::Arc;
use vernal_beans::{
    ComponentDefinition, ComponentKey, Container, Dependency, RegistryBuilder, TraitBinding,
};

#[test]
fn select_definition_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let dep = Dependency::of::<String>();
    let result = c.test_select_definition(&dep);
    assert!(result.is_err());
}

#[test]
fn select_definition_ambiguous() {
    use vernal_beans::Qualifier;
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
    let dep = Dependency::of::<String>();
    let result = c.test_select_definition(&dep);
    assert!(result.is_err());
}

#[test]
fn select_definition_ok() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let dep = Dependency::of::<String>();
    let result = c.test_select_definition(&dep);
    assert!(result.is_ok());
}

#[test]
fn select_trait_binding_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let dep = Dependency::trait_of::<dyn std::fmt::Debug + Send + Sync>();
    let result = c.test_select_trait_binding(&dep);
    assert!(result.is_err());
}

#[test]
fn resolve_definition_circular() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());

    let key = ComponentKey::of::<String>();
    let defs = c.registry_ref().definitions();
    let def = defs.first().unwrap().clone();
    let result = c.test_resolve_definition(&def, &[key]);
    assert!(matches!(
        result,
        Err(vernal_beans::ResolveError::CircularRuntime { .. })
    ));
}

#[test]
fn resolve_binding_not_found_target() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let binding = TraitBinding::new::<dyn std::any::Any + Send + Sync, String, _>(|arc| {
        arc as Arc<dyn std::any::Any + Send + Sync>
    });
    let result = c.test_resolve_binding_erased(&binding, &[]);
    assert!(result.is_err());
}
