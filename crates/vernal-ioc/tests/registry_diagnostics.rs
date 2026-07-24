//! Registry 只读诊断快照合同测试。

mod diagnostic_support;

use diagnostic_support::{Database, EnglishGreeting, Greeting, GreetingConsumer};
use vernal_ioc::{ComponentDefinition, RegistryBuilder, Scope, TraitBinding};

#[test]
fn snapshot_reuses_build_order_and_exposes_only_stable_metadata() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(
            ComponentDefinition::transient(|resolver| GreetingConsumer {
                _database: resolver.resolve().expect("declared database dependency"),
                greeting: resolver
                    .resolve_trait()
                    .expect("declared greeting dependency"),
            })
            .depends_on::<Database>()
            .depends_on_trait::<dyn Greeting>(),
        )
        .expect("consumer definition");
    registry
        .register(ComponentDefinition::singleton(|_| Database))
        .expect("database definition");
    registry
        .register(ComponentDefinition::singleton(|_| EnglishGreeting))
        .expect("greeting definition");
    registry
        .bind(TraitBinding::new::<dyn Greeting, EnglishGreeting, _>(|greeting| greeting).primary())
        .expect("greeting binding");

    let frozen = registry.build().expect("valid graph");
    let snapshot = frozen.snapshot();
    let summary = snapshot.summary();
    assert_eq!(summary.definition_count(), 3);
    assert_eq!(summary.singleton_count(), 2);
    assert_eq!(summary.transient_count(), 1);
    assert_eq!(summary.declared_dependency_count(), 2);
    assert_eq!(summary.trait_binding_count(), 1);

    let components = snapshot.components();
    assert_eq!(components.len(), 3);
    assert!(components[0].type_name().ends_with("Database"));
    assert!(components[1].type_name().ends_with("EnglishGreeting"));
    assert!(components[2].type_name().ends_with("GreetingConsumer"));
    assert_eq!(components[0].build_order(), 0);
    assert_eq!(components[2].scope(), Scope::Transient.as_str());
    assert_eq!(components[2].dependencies().len(), 2);

    let bindings = snapshot.trait_bindings();
    assert_eq!(bindings.len(), 1);
    assert!(bindings[0].trait_name().ends_with("Greeting"));
    assert!(bindings[0].target().ends_with("EnglishGreeting"));
    assert!(bindings[0].is_primary());

    let resolved: std::sync::Arc<GreetingConsumer> =
        frozen.container().resolve().expect("consumer resolution");
    assert_eq!(resolved.greeting.greet(), "hello");
}

#[test]
fn serialized_snapshot_does_not_expose_factories_or_instance_addresses() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::singleton(|_| Database))
        .expect("database definition");
    let snapshot = registry.build().expect("valid graph").snapshot();

    let first = serde_json::to_string(&snapshot).expect("serializable registry snapshot");
    let second = serde_json::to_string(&snapshot).expect("stable serialization");
    assert_eq!(first, second);
    assert!(first.contains("\"scope\":\"singleton\""));
    assert!(!first.contains("factory"));
    assert!(!first.contains("upcast"));
    assert!(!first.contains("0x"));
}
