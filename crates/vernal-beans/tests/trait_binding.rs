//! 命名、Primary 与多实现 Trait Binding 合同测试。

use std::{error::Error, sync::Arc};

use vernal_beans::{
    ComponentDefinition, DefinitionError, GraphError, Qualifier, RegistryBuilder, ResolveError,
    TraitBinding,
};
use vernal_core::BoxError;

mod trait_support;

use trait_support::{ChineseGreeting, EnglishGreeting, Greeting, GreetingConsumer};

#[test]
fn primary_named_and_all_trait_resolutions_share_native_components() {
    let chinese = Qualifier::new("chinese").expect("qualifier should be valid");
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<EnglishGreeting, _>(|_| {
            EnglishGreeting
        }))
        .expect("English implementation should register");
    builder
        .register(ComponentDefinition::singleton::<ChineseGreeting, _>(|_| {
            ChineseGreeting
        }))
        .expect("Chinese implementation should register");
    builder
        .bind(TraitBinding::new::<dyn Greeting, EnglishGreeting, _>(
            |service| service,
        ))
        .expect("English binding should register");
    builder
        .bind(
            TraitBinding::new::<dyn Greeting, ChineseGreeting, _>(|service| service)
                .qualified(chinese.clone())
                .primary(),
        )
        .expect("Chinese primary binding should register");
    builder
        .register(
            ComponentDefinition::try_singleton::<GreetingConsumer, _>(
                |resolver| -> Result<GreetingConsumer, BoxError> {
                    Ok(GreetingConsumer {
                        greeting: resolver.resolve_trait::<dyn Greeting>()?,
                    })
                },
            )
            .depends_on_trait::<dyn Greeting>(),
        )
        .expect("consumer should register");
    builder
        .register(
            ComponentDefinition::try_singleton::<Vec<Arc<dyn Greeting>>, _>(
                |resolver| -> Result<Vec<Arc<dyn Greeting>>, BoxError> {
                    Ok(resolver.resolve_all_traits::<dyn Greeting>()?)
                },
            )
            .depends_on_all_traits::<dyn Greeting>(),
        )
        .expect("collection consumer should register");

    let registry = builder.build().expect("trait graph should be valid");
    let container = registry.container();
    let consumer = container
        .resolve::<GreetingConsumer>()
        .expect("consumer should resolve");
    let selected = container
        .resolve_trait::<dyn Greeting>()
        .expect("primary implementation should resolve");
    let named = container
        .resolve_qualified_trait::<dyn Greeting>(&chinese)
        .expect("named implementation should resolve");
    let all = container
        .resolve::<Vec<Arc<dyn Greeting>>>()
        .expect("all implementations should resolve");

    assert_eq!(consumer.greeting.message(), "你好");
    assert_eq!(selected.message(), "你好");
    assert_eq!(named.message(), "你好");
    assert_eq!(
        all.iter()
            .map(|service| service.message())
            .collect::<Vec<_>>(),
        ["hello", "你好"]
    );
}

#[test]
fn all_trait_dependency_allows_an_empty_implementation_set() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(
            ComponentDefinition::try_singleton::<Vec<Arc<dyn Greeting>>, _>(
                |resolver| -> Result<Vec<Arc<dyn Greeting>>, BoxError> {
                    Ok(resolver.resolve_all_traits::<dyn Greeting>()?)
                },
            )
            .depends_on_all_traits::<dyn Greeting>(),
        )
        .expect("empty collection consumer should register");

    let services = builder
        .build()
        .expect("empty all-trait dependency should be valid")
        .container()
        .resolve::<Vec<Arc<dyn Greeting>>>()
        .expect("empty collection should resolve");

    assert!(services.is_empty());
}

#[test]
fn graph_rejects_an_ambiguous_unqualified_trait_dependency() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<EnglishGreeting, _>(|_| {
            EnglishGreeting
        }))
        .expect("English implementation should register");
    builder
        .register(ComponentDefinition::singleton::<ChineseGreeting, _>(|_| {
            ChineseGreeting
        }))
        .expect("Chinese implementation should register");
    builder
        .bind_all([
            TraitBinding::new::<dyn Greeting, EnglishGreeting, _>(|service| service),
            TraitBinding::new::<dyn Greeting, ChineseGreeting, _>(|service| service),
        ])
        .expect("multiple non-primary bindings are valid");
    builder
        .register(
            ComponentDefinition::singleton::<GreetingConsumer, _>(|_| {
                unreachable!("invalid graph must not construct the consumer")
            })
            .depends_on_trait::<dyn Greeting>(),
        )
        .expect("consumer definition should register");

    let error = builder
        .build()
        .expect_err("ambiguous trait dependency must fail before runtime");
    let GraphError::AmbiguousDependency { path, candidates } = error else {
        panic!("expected ambiguous dependency error");
    };

    assert_eq!(path.len(), 2);
    assert_eq!(candidates.len(), 2);
    assert!(path[1].contains("Greeting"));
}

#[test]
fn binding_target_must_be_a_registered_component_definition() {
    let mut builder = RegistryBuilder::new();
    builder
        .bind(TraitBinding::new::<dyn Greeting, EnglishGreeting, _>(
            |service| service,
        ))
        .expect("binding syntax should be valid");

    let error = builder
        .build()
        .expect_err("missing binding target must fail graph validation");

    assert!(matches!(
        error,
        GraphError::MissingTraitBindingTarget { .. }
    ));
}

#[test]
fn primary_conflict_rejects_the_complete_binding_batch_atomically() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<EnglishGreeting, _>(|_| {
            EnglishGreeting
        }))
        .expect("English implementation should register");
    builder
        .register(ComponentDefinition::singleton::<ChineseGreeting, _>(|_| {
            ChineseGreeting
        }))
        .expect("Chinese implementation should register");
    builder
        .bind(TraitBinding::new::<dyn Greeting, EnglishGreeting, _>(|service| service).primary())
        .expect("first primary should register");

    let error = builder
        .bind_all([
            TraitBinding::new::<dyn Greeting, ChineseGreeting, _>(|service| service).primary(),
        ])
        .expect_err("second primary should reject the entire batch");
    assert!(matches!(
        error,
        DefinitionError::MultiplePrimaryTraitBindings { .. }
    ));

    let registry = builder
        .build()
        .expect("failed batch must leave the previous registry valid");
    assert_eq!(registry.bindings().len(), 1);
    assert_eq!(
        registry
            .container()
            .resolve_trait::<dyn Greeting>()
            .expect("original primary should remain")
            .message(),
        "hello"
    );
}

#[test]
fn component_and_trait_bundle_is_atomic_across_both_registration_kinds() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<EnglishGreeting, _>(|_| {
            EnglishGreeting
        }))
        .expect("English implementation should register");
    builder
        .bind(TraitBinding::new::<dyn Greeting, EnglishGreeting, _>(|service| service).primary())
        .expect("English primary should register");

    let error = builder
        .register_bundle(
            [ComponentDefinition::singleton::<ChineseGreeting, _>(|_| {
                ChineseGreeting
            })],
            [TraitBinding::new::<dyn Greeting, ChineseGreeting, _>(|service| service).primary()],
        )
        .expect_err("binding conflict must reject definitions and bindings together");
    assert!(matches!(
        error,
        DefinitionError::MultiplePrimaryTraitBindings { .. }
    ));

    let registry = builder
        .build()
        .expect("failed bundle must leave the previous graph valid");
    assert_eq!(registry.definitions().len(), 1);
    assert_eq!(registry.bindings().len(), 1);
    assert!(matches!(
        registry.container().resolve::<ChineseGreeting>(),
        Err(ResolveError::NotFound { .. })
    ));
}

#[test]
fn factory_cannot_resolve_an_undeclared_trait_dependency() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<EnglishGreeting, _>(|_| {
            EnglishGreeting
        }))
        .expect("English implementation should register");
    builder
        .bind(TraitBinding::new::<dyn Greeting, EnglishGreeting, _>(
            |service| service,
        ))
        .expect("English binding should register");
    builder
        .register(ComponentDefinition::try_singleton::<GreetingConsumer, _>(
            |resolver| -> Result<GreetingConsumer, BoxError> {
                Ok(GreetingConsumer {
                    greeting: resolver.resolve_trait::<dyn Greeting>()?,
                })
            },
        ))
        .expect("consumer should register");

    let result = builder
        .build()
        .expect("graph without declared edge can freeze")
        .container()
        .resolve::<GreetingConsumer>();
    let Err(error) = result else {
        panic!("hidden trait dependency must fail");
    };

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
fn qualified_trait_name_must_select_only_one_target() -> Result<(), Box<dyn Error>> {
    let api = Qualifier::new("api")?;
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<EnglishGreeting, _>(|_| {
        EnglishGreeting
    }))?;
    builder.register(ComponentDefinition::singleton::<ChineseGreeting, _>(|_| {
        ChineseGreeting
    }))?;
    builder.bind(
        TraitBinding::new::<dyn Greeting, EnglishGreeting, _>(|service| service)
            .qualified(api.clone()),
    )?;

    let error = builder
        .bind(
            TraitBinding::new::<dyn Greeting, ChineseGreeting, _>(|service| service).qualified(api),
        )
        .expect_err("a named trait key must not point to two targets");

    assert!(matches!(
        error,
        DefinitionError::DuplicateQualifiedTraitBinding { .. }
    ));
    Ok(())
}

#[test]
fn trait_binding_can_target_a_qualified_concrete_component() -> Result<(), Box<dyn Error>> {
    let formal = Qualifier::new("formal")?;
    let api = Qualifier::new("api")?;
    let mut builder = RegistryBuilder::new();
    builder.register(
        ComponentDefinition::singleton::<EnglishGreeting, _>(|_| EnglishGreeting)
            .qualified(formal.clone()),
    )?;
    builder.bind(
        TraitBinding::new::<dyn Greeting, EnglishGreeting, _>(|service| service)
            .qualified(api.clone())
            .target_qualified(formal.clone()),
    )?;

    let container = builder.build()?.container();
    let concrete = container.resolve_qualified::<EnglishGreeting>(&formal)?;
    let concrete_as_trait: Arc<dyn Greeting> = concrete;
    let bound = container.resolve_qualified_trait::<dyn Greeting>(&api)?;

    assert!(Arc::ptr_eq(&concrete_as_trait, &bound));
    Ok(())
}

#[test]
fn trait_dependency_participates_in_cycle_detection() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(
            ComponentDefinition::singleton::<EnglishGreeting, _>(|_| EnglishGreeting)
                .depends_on::<GreetingConsumer>(),
        )
        .expect("English definition should register");
    builder
        .bind(TraitBinding::new::<dyn Greeting, EnglishGreeting, _>(
            |service| service,
        ))
        .expect("English binding should register");
    builder
        .register(
            ComponentDefinition::singleton::<GreetingConsumer, _>(|_| {
                unreachable!("cyclic graph must not construct the consumer")
            })
            .depends_on_trait::<dyn Greeting>(),
        )
        .expect("consumer definition should register");

    let error = builder
        .build()
        .expect_err("trait edge must participate in cycle detection");
    let GraphError::Cycle { path } = error else {
        panic!("expected cycle error");
    };

    assert_eq!(path.len(), 3);
    assert_eq!(path.first(), path.last());
}
