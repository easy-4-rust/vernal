//! Contract tests for the typed `IoC` kernel.

use std::{
    error::Error,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};

use vernal_core::BoxError;
use vernal_beans::{
    ComponentDefinition, DefinitionError, GraphError, Qualifier, Registry, RegistryBuilder,
    ResolveError,
};

#[derive(Debug)]
struct Settings {
    application_name: &'static str,
}

#[derive(Debug)]
struct Service {
    settings: Arc<Settings>,
}

#[test]
fn resolves_declared_dependencies_and_caches_singletons() {
    let settings_calls = Arc::new(AtomicUsize::new(0));
    let observed_calls = Arc::clone(&settings_calls);
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Settings, _>(move |_| {
            observed_calls.fetch_add(1, Ordering::SeqCst);
            Settings {
                application_name: "vernal",
            }
        }))
        .expect("settings definition should be valid");
    builder
        .register(
            ComponentDefinition::try_singleton::<Service, _>(
                |resolver| -> Result<Service, BoxError> {
                    Ok(Service {
                        settings: resolver.resolve::<Settings>()?,
                    })
                },
            )
            .depends_on::<Settings>(),
        )
        .expect("service definition should be valid");

    let registry = builder.build().expect("graph should be valid");
    let container = registry.container();
    container.warm_up().expect("singletons should construct");

    let first = container
        .resolve::<Service>()
        .expect("service should resolve");
    let second = container
        .resolve::<Service>()
        .expect("service should resolve again");

    assert!(Arc::ptr_eq(&first, &second));
    assert!(Arc::ptr_eq(&first.settings, &second.settings));
    assert_eq!(first.settings.application_name, "vernal");
    assert_eq!(settings_calls.load(Ordering::SeqCst), 1);
}

#[derive(Debug)]
struct Marker;

#[test]
fn singleton_is_once_per_container_even_under_concurrency() {
    let calls = Arc::new(AtomicUsize::new(0));
    let observed_calls = Arc::clone(&calls);
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Marker, _>(move |_| {
            observed_calls.fetch_add(1, Ordering::SeqCst);
            Marker
        }))
        .expect("marker definition should be valid");
    let registry = builder.build().expect("graph should be valid");

    let first_container = Arc::new(registry.container());
    let second_container = Arc::new(registry.container());
    let threads: Vec<_> = (0..32)
        .map(|index| {
            let container = if index % 2 == 0 {
                Arc::clone(&first_container)
            } else {
                Arc::clone(&second_container)
            };
            thread::spawn(move || {
                (
                    index % 2,
                    container
                        .resolve::<Marker>()
                        .expect("concurrent singleton should resolve"),
                )
            })
        })
        .collect();
    let mut first_instances = Vec::new();
    let mut second_instances = Vec::new();
    for handle in threads {
        let (group, instance) = handle.join().expect("worker should finish");
        if group == 0 {
            first_instances.push(instance);
        } else {
            second_instances.push(instance);
        }
    }

    for instance in &first_instances[1..] {
        assert!(Arc::ptr_eq(&first_instances[0], instance));
    }
    for instance in &second_instances[1..] {
        assert!(Arc::ptr_eq(&second_instances[0], instance));
    }
    assert!(!Arc::ptr_eq(&first_instances[0], &second_instances[0]));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn tokio_runtime_handle_can_be_used_as_a_native_shared_component() {
    let runtime = Arc::new(tokio::runtime::Handle::current());
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_arc(Arc::clone(&runtime)))
        .expect("Tokio runtime handle definition should be valid");
    let registry = builder.build().expect("graph should be valid");

    let first = registry
        .container()
        .resolve::<tokio::runtime::Handle>()
        .expect("runtime handle should resolve");
    let second = registry
        .container()
        .resolve::<tokio::runtime::Handle>()
        .expect("runtime handle should be shared across containers");

    assert!(Arc::ptr_eq(&runtime, &first));
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(
        first
            .spawn(async { "vernal-tokio" })
            .await
            .expect("Tokio task should finish"),
        "vernal-tokio"
    );
}

#[test]
fn owned_native_value_can_be_registered_without_a_wrapper_type() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(String::from(
            "vernal-native-component",
        )))
        .expect("native String definition should be valid");
    let container = builder.build().expect("graph should be valid").container();

    let value = container
        .resolve::<String>()
        .expect("native String should resolve");

    assert_eq!(value.as_str(), "vernal-native-component");
}

#[derive(Debug)]
struct Sequence(usize);

#[test]
fn transient_scope_constructs_on_every_resolution() {
    let sequence = Arc::new(AtomicUsize::new(0));
    let observed_sequence = Arc::clone(&sequence);
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::transient::<Sequence, _>(move |_| {
            Sequence(observed_sequence.fetch_add(1, Ordering::SeqCst))
        }))
        .expect("transient definition should be valid");
    let container = builder.build().expect("graph should be valid").container();

    let first = container
        .resolve::<Sequence>()
        .expect("first transient should resolve");
    let second = container
        .resolve::<Sequence>()
        .expect("second transient should resolve");

    assert_eq!(first.0, 0);
    assert_eq!(second.0, 1);
    assert!(!Arc::ptr_eq(&first, &second));
}

struct MissingRoot;
struct MissingMiddle;
struct MissingLeaf;

#[test]
fn missing_dependency_reports_the_complete_path() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(
            ComponentDefinition::singleton::<MissingRoot, _>(|_| MissingRoot)
                .depends_on::<MissingMiddle>(),
        )
        .expect("root definition should be valid");
    builder
        .register(
            ComponentDefinition::singleton::<MissingMiddle, _>(|_| MissingMiddle)
                .depends_on::<MissingLeaf>(),
        )
        .expect("middle definition should be valid");

    let error = builder.build().expect_err("missing leaf must fail");
    let GraphError::MissingDependency { path } = error else {
        panic!("expected missing dependency error");
    };
    assert_eq!(path.len(), 3);
    assert!(path[0].ends_with("MissingRoot"));
    assert!(path[1].ends_with("MissingMiddle"));
    assert!(path[2].ends_with("MissingLeaf"));
}

struct Port;
struct PortConsumer;

#[test]
fn unqualified_dependency_rejects_ambiguous_candidates() {
    let primary = Qualifier::new("primary").expect("qualifier should be valid");
    let secondary = Qualifier::new("secondary").expect("qualifier should be valid");
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Port, _>(|_| Port).qualified(primary))
        .expect("primary port should be valid");
    builder
        .register(ComponentDefinition::singleton::<Port, _>(|_| Port).qualified(secondary))
        .expect("secondary port should be valid");
    builder
        .register(
            ComponentDefinition::singleton::<PortConsumer, _>(|_| PortConsumer)
                .depends_on::<Port>(),
        )
        .expect("consumer should be valid");

    let error = builder.build().expect_err("ambiguous port must fail");
    let GraphError::AmbiguousDependency { path, candidates } = error else {
        panic!("expected ambiguous dependency error");
    };
    assert_eq!(path.len(), 2);
    assert_eq!(candidates.len(), 2);
    assert!(candidates.iter().any(|value| value.ends_with("@primary")));
    assert!(candidates.iter().any(|value| value.ends_with("@secondary")));
}

struct CycleNode;

#[test]
fn cycle_error_contains_a_closed_readable_path() {
    let left = Qualifier::new("left").expect("qualifier should be valid");
    let right = Qualifier::new("right").expect("qualifier should be valid");
    let mut builder = RegistryBuilder::new();
    builder
        .register(
            ComponentDefinition::singleton::<CycleNode, _>(|_| CycleNode)
                .qualified(left.clone())
                .depends_on_qualified::<CycleNode>(right.clone()),
        )
        .expect("left node should be valid");
    builder
        .register(
            ComponentDefinition::singleton::<CycleNode, _>(|_| CycleNode)
                .qualified(right)
                .depends_on_qualified::<CycleNode>(left),
        )
        .expect("right node should be valid");

    let error = builder.build().expect_err("cycle must fail");
    let GraphError::Cycle { path } = error else {
        panic!("expected cycle error");
    };
    assert_eq!(path.len(), 3);
    assert_eq!(path.first(), path.last());
}

#[test]
fn graph_plans_one_thousand_nodes_deterministically() {
    fn build_registry() -> Registry {
        let mut builder = RegistryBuilder::new();
        for index in (0..1_000).rev() {
            let qualifier = Qualifier::new(format!("node-{index:04}")).expect("valid qualifier");
            let mut definition =
                ComponentDefinition::singleton::<CycleNode, _>(|_| CycleNode).qualified(qualifier);
            if index > 0 {
                let dependency = Qualifier::new(format!("node-{:04}", index - 1))
                    .expect("valid dependency qualifier");
                definition = definition.depends_on_qualified::<CycleNode>(dependency);
            }
            builder
                .register(definition)
                .expect("node definition should be unique");
        }
        builder.build().expect("large graph should be valid")
    }

    let first = build_registry();
    let second = build_registry();
    let first_keys: Vec<String> = first
        .plan()
        .keys()
        .iter()
        .map(ToString::to_string)
        .collect();
    let second_keys: Vec<String> = second
        .plan()
        .keys()
        .iter()
        .map(ToString::to_string)
        .collect();

    assert_eq!(first.plan().len(), 1_000);
    assert_eq!(first_keys, second_keys);
    assert!(first_keys[0].ends_with("@node-0000"));
    assert!(first_keys[999].ends_with("@node-0999"));
}

#[test]
fn factory_cannot_resolve_an_undeclared_dependency() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Settings, _>(|_| {
            Settings {
                application_name: "vernal",
            }
        }))
        .expect("settings should be valid");
    builder
        .register(ComponentDefinition::try_singleton::<Service, _>(
            |resolver| -> Result<Service, BoxError> {
                Ok(Service {
                    settings: resolver.resolve::<Settings>()?,
                })
            },
        ))
        .expect("service definition should be valid");
    let container = builder
        .build()
        .expect("declared graph itself is valid")
        .container();

    let error = container
        .resolve::<Service>()
        .expect_err("hidden dependency must fail");
    assert!(matches!(error, ResolveError::Construction { .. }));
    assert!(
        error
            .source()
            .expect("construction error should preserve source")
            .to_string()
            .contains("undeclared dependency")
    );
}

#[test]
fn duplicate_definition_and_invalid_qualifier_are_rejected() {
    let qualifier_error = Qualifier::new(" invalid ").expect_err("whitespace must be rejected");
    assert!(matches!(
        qualifier_error,
        DefinitionError::InvalidQualifier { .. }
    ));

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Marker, _>(|_| Marker))
        .expect("first marker should be valid");
    let duplicate = builder
        .register(ComponentDefinition::singleton::<Marker, _>(|_| Marker))
        .expect_err("duplicate marker must fail");
    assert!(matches!(
        duplicate,
        DefinitionError::DuplicateDefinition { .. }
    ));
}

#[test]
fn batch_registration_is_atomic_when_any_definition_conflicts() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::shared_value(1_u8))
        .expect("initial definition");

    let error = registry
        .register_all([
            ComponentDefinition::shared_value(2_u16),
            ComponentDefinition::shared_value(3_u8),
        ])
        .expect_err("batch must reject an existing component key");

    assert!(matches!(error, DefinitionError::DuplicateDefinition { .. }));
    assert_eq!(
        registry.len(),
        1,
        "failed batch must not retain its valid prefix"
    );
    assert!(matches!(
        registry
            .build()
            .expect("original graph remains valid")
            .container()
            .resolve::<u16>(),
        Err(ResolveError::NotFound { .. })
    ));
}
