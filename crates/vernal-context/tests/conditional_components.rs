//! Environment/Profile 条件组件装配合同测试。

use std::{
    error::Error,
    fmt::{Debug, Display},
    io,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use vernal_context::{
    ApplicationBuildError, ComponentCondition, ConditionError, ConditionalComponentModule,
    Lifecycle, LifecycleFuture, MapPropertySource, PredicateCondition, ProfileCondition,
    PropertyCondition, VernalApplicationBuilder,
};
use vernal_core::BoxError;
use vernal_ioc::{ComponentDefinition, GraphError, TraitBinding};

/// 同时用于验证条件定义与条件生命周期登记的一项普通业务组件。
struct ConditionalService {
    lifecycle_calls: Arc<AtomicUsize>,
    _dependency: Option<Arc<u16>>,
}

impl Lifecycle for ConditionalService {
    fn initialize(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.lifecycle_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
    }

    fn start(&self, _cancellation: tokio_util::sync::CancellationToken) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.lifecycle_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
    }
}

#[tokio::test]
async fn profile_condition_commits_definition_and_lifecycle_as_one_module() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut application = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    application
        .environment()
        .active_profile("production")
        .expect("active profile");

    let component_calls = Arc::clone(&calls);
    let mut production = ConditionalComponentModule::new(
        "production-service",
        ProfileCondition::any(["production"]).expect("valid profile condition"),
    );
    production
        .register(ComponentDefinition::singleton(move |_| {
            ConditionalService {
                lifecycle_calls: Arc::clone(&component_calls),
                _dependency: None,
            }
        }))
        .lifecycle::<ConditionalService>();
    application
        .register_conditional(production)
        .expect("valid conditional module");

    // 未命中模块中的生命周期登记必须和定义一起排除，不能让 Context 构建阶段
    // 再去解析一个不存在的组件。
    let mut test_only = ConditionalComponentModule::new(
        "test-only-number",
        ProfileCondition::any(["test"]).expect("valid profile condition"),
    );
    test_only.register(ComponentDefinition::shared_value(41_u16));
    application
        .register_conditional(test_only)
        .expect("valid unmatched module");

    let context = application.build().expect("conditional application");
    assert!(context.container().resolve::<ConditionalService>().is_ok());
    assert!(context.container().resolve::<u16>().is_err());

    context.refresh().await.expect("refresh");
    context.start().await.expect("start");
    assert_eq!(calls.load(Ordering::SeqCst), 2);

    let report = context.startup_report().await;
    assert_eq!(report.condition_evaluations().len(), 2);
    assert_eq!(
        report.condition_evaluations()[0].module(),
        "production-service"
    );
    assert_eq!(report.condition_evaluations()[0].condition(), "profile.any");
    assert!(report.condition_evaluations()[0].matched());
    assert_eq!(report.condition_evaluations()[0].lifecycle_count(), 1);
    assert!(!report.condition_evaluations()[1].matched());
    assert_eq!(report.registry().summary().definition_count(), 12);

    context.close().await.expect("close");
}

#[tokio::test]
async fn property_conditions_use_resolved_values_without_leaking_keys_or_values() {
    let secret_key = "feature.private-mode";
    let secret_value = "sensitive-enablement-token";
    let mut application = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    application
        .environment()
        .add_last(Arc::new(
            MapPropertySource::new(
                "application",
                [
                    (secret_key, "${feature.default}"),
                    ("feature.default", secret_value),
                ],
            )
            .expect("property source"),
        ))
        .expect("environment source");

    let condition =
        PropertyCondition::having_value(secret_key, secret_value).expect("valid property");
    let debug = format!("{condition:?}");
    assert!(!debug.contains(secret_key));
    assert!(!debug.contains(secret_value));

    let mut enabled = ConditionalComponentModule::new("private-feature", condition);
    enabled
        .register(ComponentDefinition::shared_value(42_u32))
        .bind(TraitBinding::new::<dyn Display + Send + Sync, u32, _>(
            |value| value,
        ));
    application
        .register_conditional(enabled)
        .expect("enabled module");

    let mut absent = ConditionalComponentModule::new(
        "missing-feature",
        PropertyCondition::present("feature.missing").expect("valid missing key"),
    );
    absent
        .register(ComponentDefinition::shared_value(64_u64))
        .bind(TraitBinding::new::<dyn Debug + Send + Sync, u64, _>(
            |value| value,
        ));
    application
        .register_conditional(absent)
        .expect("unmatched module");

    let context = application.build().expect("conditional application");
    assert_eq!(
        *context.container().resolve::<u32>().expect("enabled value"),
        42
    );
    assert_eq!(
        context
            .container()
            .resolve_trait::<dyn Display + Send + Sync>()
            .expect("enabled trait binding")
            .to_string(),
        "42"
    );
    assert!(context.container().resolve::<u64>().is_err());
    assert!(
        context
            .container()
            .resolve_trait::<dyn Debug + Send + Sync>()
            .is_err()
    );

    let report = context.startup_report().await;
    assert!(report.condition_evaluations()[0].matched());
    assert!(!report.condition_evaluations()[1].matched());
    assert_eq!(
        report.condition_evaluations()[0].condition(),
        "property.equals"
    );
    assert_eq!(report.condition_evaluations()[0].trait_binding_count(), 1);
    assert_eq!(report.condition_evaluations()[1].trait_binding_count(), 1);

    let json = serde_json::to_string(&report).expect("serializable report");
    assert!(!json.contains(secret_key));
    assert!(!json.contains(secret_value));
    assert!(!json.contains("feature.missing"));
}

#[tokio::test]
async fn false_condition_keeps_missing_dependency_visible_to_graph_validation() {
    let mut application = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    application
        .register(
            ComponentDefinition::try_singleton(|resolver| {
                Ok(ConditionalService {
                    lifecycle_calls: Arc::new(AtomicUsize::new(0)),
                    _dependency: Some(resolver.resolve::<u16>()?),
                })
            })
            .depends_on::<u16>(),
        )
        .expect("dependent definition");

    let mut excluded = ConditionalComponentModule::new(
        "excluded-dependency",
        PropertyCondition::present("feature.number").expect("valid property key"),
    );
    excluded.register(ComponentDefinition::shared_value(7_u16));
    application
        .register_conditional(excluded)
        .expect("conditional dependency");

    let Err(error) = application.build() else {
        panic!("missing conditional dependency must fail closed");
    };
    assert!(matches!(
        error,
        ApplicationBuildError::Graph {
            source: GraphError::MissingDependency { .. }
        }
    ));
}

#[tokio::test]
async fn module_declaration_rejects_empty_invalid_and_duplicate_diagnostics() {
    let mut application = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let empty = ConditionalComponentModule::new(
        "empty-module",
        ProfileCondition::any(["default"]).expect("valid profile"),
    );
    assert!(matches!(
        application.register_conditional(empty),
        Err(ConditionError::EmptyModule {
            name: "empty-module"
        })
    ));

    let mut invalid_condition = ConditionalComponentModule::new(
        "invalid-condition",
        PredicateCondition::new("bad name", |_| Ok(true)),
    );
    invalid_condition.register(ComponentDefinition::shared_value(1_u8));
    assert!(matches!(
        application.register_conditional(invalid_condition),
        Err(ConditionError::InvalidConditionName {
            module: "invalid-condition",
            ..
        })
    ));

    let mut first = ConditionalComponentModule::new(
        "unique-module",
        PropertyCondition::missing("feature.first").expect("valid property"),
    );
    first.register(ComponentDefinition::shared_value(1_u8));
    application
        .register_conditional(first)
        .expect("first module");

    let mut duplicate = ConditionalComponentModule::new(
        "unique-module",
        PropertyCondition::missing("feature.second").expect("valid property"),
    );
    duplicate.register(ComponentDefinition::shared_value(2_u16));
    assert!(matches!(
        application.register_conditional(duplicate),
        Err(ConditionError::DuplicateModule {
            name: "unique-module"
        })
    ));
}

#[tokio::test]
async fn custom_condition_failure_is_redacted_but_preserves_explicit_source_chain() {
    let mut application = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    let condition = PredicateCondition::new("remote.feature", |_| {
        Err(Box::new(io::Error::other("secret-remote-configuration-response")) as BoxError)
    });
    let mut module = ConditionalComponentModule::new("remote-feature", condition);
    module.register(ComponentDefinition::shared_value(1_u64));
    application
        .register_conditional(module)
        .expect("valid conditional module");

    let Err(error) = application.build() else {
        panic!("condition evaluation must fail");
    };
    let display = error.to_string();
    let debug = format!("{error:?}");
    assert!(!display.contains("secret-remote-configuration-response"));
    assert!(!debug.contains("secret-remote-configuration-response"));

    let condition_error = error.source().expect("condition error");
    let root = condition_error.source().expect("custom predicate source");
    assert!(
        root.to_string()
            .contains("secret-remote-configuration-response")
    );
}

#[test]
fn built_in_conditions_cover_profile_sets_missing_and_negative_value_semantics() {
    let mut environment = vernal_context::ApplicationEnvironment::builder();
    environment
        .active_profile("production")
        .expect("production profile")
        .active_profile("observability")
        .expect("observability profile")
        .add_last(Arc::new(
            MapPropertySource::new("application", [("feature.mode", "safe")])
                .expect("property source"),
        ))
        .expect("environment source");
    let environment = environment.build();

    assert!(
        ProfileCondition::all(["production", "observability"])
            .expect("condition")
            .matches(&environment)
            .expect("evaluation")
    );
    assert!(
        ProfileCondition::none(["test", "development"])
            .expect("condition")
            .matches(&environment)
            .expect("evaluation")
    );
    assert!(
        !ProfileCondition::all(["production", "test"])
            .expect("condition")
            .matches(&environment)
            .expect("evaluation")
    );
    assert!(
        PropertyCondition::present("feature.mode")
            .expect("condition")
            .matches(&environment)
            .expect("evaluation")
    );
    assert!(
        PropertyCondition::missing("feature.absent")
            .expect("condition")
            .matches(&environment)
            .expect("evaluation")
    );
    assert!(
        PropertyCondition::not_having_value("feature.mode", "unsafe")
            .expect("condition")
            .matches(&environment)
            .expect("evaluation")
    );
    assert!(
        PropertyCondition::having_value_or_missing("feature.absent", "enabled")
            .expect("condition")
            .matches(&environment)
            .expect("evaluation")
    );
    assert!(
        !PropertyCondition::not_having_value("feature.absent", "unsafe")
            .expect("condition")
            .matches(&environment)
            .expect("evaluation")
    );
}
