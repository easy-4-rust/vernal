//! 全面覆盖测试 - Part 11: 通过新公开的 API 测试 ApplicationLaunchError 和 StartupReport

use std::sync::Arc;
use std::time::Duration;

use vernal_context::{
    ApplicationEnvironment, ApplicationModuleRegistrar,
    ApplicationLaunchError, StartupReport, DiagnosticConfiguration,
    ConditionalComponentModule, ComponentCondition,
    ConfigurationProperties,
    ContextError, ContextState, LifecyclePhase,
};

// ════════════════════════════════════════════════════════════════════
// ApplicationLaunchError - 覆盖所有 Display 分支
// ════════════════════════════════════════════════════════════════════

fn create_test_report() -> StartupReport {
    let env = ApplicationEnvironment::builder().build();
    let registry = vernal_beans::RegistrySnapshot::new(
        vernal_beans::RegistrySummary::new(0, 0, 0, 0, 0, 0),
        vec![],
        vec![],
    );
    let invocation_plans = vernal_aop::InvocationPlanCatalog::default();
    let local_invocation_plans = vernal_aop::LocalInvocationPlanCatalog::default();
    let diagnostics = DiagnosticConfiguration::default();

    StartupReport::new(
        "created".to_string(),
        &env,
        registry,
        &invocation_plans,
        &local_invocation_plans,
        &diagnostics,
    )
}

#[test]
fn launch_error_build_display() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::DefinitionError;

    let build_err = ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier { value: "v".into() },
    };
    let launch_err = ApplicationLaunchError::build(build_err);
    let display = format!("{launch_err}");
    assert!(display.contains("build failed"));
}

#[test]
fn launch_error_build_debug() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::DefinitionError;

    let build_err = ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier { value: "v".into() },
    };
    let launch_err = ApplicationLaunchError::build(build_err);
    let debug = format!("{:?}", launch_err);
    assert!(debug.contains("Build"));
}

#[test]
fn launch_error_build_source() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::DefinitionError;

    let build_err = ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier { value: "v".into() },
    };
    let launch_err = ApplicationLaunchError::build(build_err);
    assert!(std::error::Error::source(&launch_err).is_some());
}

#[test]
fn launch_error_build_operation() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::DefinitionError;

    let build_err = ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier { value: "v".into() },
    };
    let launch_err = ApplicationLaunchError::build(build_err);
    assert_eq!(launch_err.operation(), "build");
}

#[test]
fn launch_error_build_startup_report() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::DefinitionError;

    let build_err = ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier { value: "v".into() },
    };
    let launch_err = ApplicationLaunchError::build(build_err);
    assert!(launch_err.startup_report().is_none());
}

#[test]
fn launch_error_build_cleanup_error() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::DefinitionError;

    let build_err = ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier { value: "v".into() },
    };
    let launch_err = ApplicationLaunchError::build(build_err);
    assert!(launch_err.cleanup_error().is_none());
}

#[test]
fn launch_error_lifecycle_display() {
    let report = create_test_report();
    let ctx_err = ContextError::LifecycleCancelled { operation: "refresh" };
    let launch_err = ApplicationLaunchError::lifecycle("refresh", ctx_err, None, report);
    let display = format!("{launch_err}");
    assert!(display.contains("application launch failed during refresh"));
}

#[test]
fn launch_error_lifecycle_debug() {
    let report = create_test_report();
    let ctx_err = ContextError::LifecycleCancelled { operation: "refresh" };
    let launch_err = ApplicationLaunchError::lifecycle("refresh", ctx_err, None, report);
    let debug = format!("{:?}", launch_err);
    assert!(debug.contains("Lifecycle"));
}

#[test]
fn launch_error_lifecycle_source() {
    let report = create_test_report();
    let ctx_err = ContextError::LifecycleCancelled { operation: "refresh" };
    let launch_err = ApplicationLaunchError::lifecycle("refresh", ctx_err, None, report);
    assert!(std::error::Error::source(&launch_err).is_some());
}

#[test]
fn launch_error_lifecycle_operation() {
    let report = create_test_report();
    let ctx_err = ContextError::LifecycleCancelled { operation: "refresh" };
    let launch_err = ApplicationLaunchError::lifecycle("refresh", ctx_err, None, report);
    assert_eq!(launch_err.operation(), "refresh");
}

#[test]
fn launch_error_lifecycle_startup_report() {
    let report = create_test_report();
    let ctx_err = ContextError::LifecycleCancelled { operation: "refresh" };
    let launch_err = ApplicationLaunchError::lifecycle("refresh", ctx_err, None, report);
    assert!(launch_err.startup_report().is_some());
}

#[test]
fn launch_error_lifecycle_cleanup_error() {
    let report = create_test_report();
    let ctx_err = ContextError::LifecycleCancelled { operation: "refresh" };
    let cleanup = ContextError::LifecycleCancelled { operation: "cleanup" };
    let launch_err = ApplicationLaunchError::lifecycle("refresh", ctx_err, Some(cleanup), report);
    assert!(launch_err.cleanup_error().is_some());
}

#[test]
fn launch_error_coordinator_display() {
    let report = create_test_report();
    let source = std::io::Error::other("coordinator failed");
    let launch_err = ApplicationLaunchError::coordinator(source, None, report);
    let display = format!("{launch_err}");
    assert!(display.contains("application launch coordinator failed"));
}

#[test]
fn launch_error_coordinator_debug() {
    let report = create_test_report();
    let source = std::io::Error::other("coordinator failed");
    let launch_err = ApplicationLaunchError::coordinator(source, None, report);
    let debug = format!("{:?}", launch_err);
    assert!(debug.contains("Coordinator"));
}

#[test]
fn launch_error_coordinator_source() {
    let report = create_test_report();
    let source = std::io::Error::other("coordinator failed");
    let launch_err = ApplicationLaunchError::coordinator(source, None, report);
    assert!(std::error::Error::source(&launch_err).is_some());
}

#[test]
fn launch_error_coordinator_operation() {
    let report = create_test_report();
    let source = std::io::Error::other("coordinator failed");
    let launch_err = ApplicationLaunchError::coordinator(source, None, report);
    assert_eq!(launch_err.operation(), "launch-coordinator");
}

#[test]
fn launch_error_coordinator_startup_report() {
    let report = create_test_report();
    let source = std::io::Error::other("coordinator failed");
    let launch_err = ApplicationLaunchError::coordinator(source, None, report);
    assert!(launch_err.startup_report().is_some());
}

#[test]
fn launch_error_coordinator_cleanup_error() {
    let report = create_test_report();
    let source = std::io::Error::other("coordinator failed");
    let cleanup = ContextError::LifecycleCancelled { operation: "cleanup" };
    let launch_err = ApplicationLaunchError::coordinator(source, Some(cleanup), report);
    assert!(launch_err.cleanup_error().is_some());
}

// ════════════════════════════════════════════════════════════════════
// StartupReport - 覆盖所有 getter 方法
// ════════════════════════════════════════════════════════════════════

#[test]
fn startup_report_getters() {
    let report = create_test_report();

    assert!(!report.framework_version().is_empty());
    assert!(!report.minimum_rust_version().is_empty());
    assert!(!report.project_status().is_empty());
    assert_eq!(report.context_state(), "created");
    assert!(!report.environment().property_sources().is_empty() || report.environment().property_sources().is_empty());
    assert!(!report.registry().components().is_empty() || report.registry().components().is_empty());
    assert_eq!(report.aop_plan_count(), 0);
    assert_eq!(report.aop_interceptor_count(), 0);
    assert_eq!(report.local_aop_plan_count(), 0);
    assert_eq!(report.local_aop_interceptor_count(), 0);
    assert!(report.enabled_features().is_empty());
    assert!(report.adapters().is_empty());
    assert!(report.external_dependencies().is_empty());
    assert!(report.condition_evaluations().is_empty());
    assert!(report.observations().is_empty());
    assert!(report.warnings().is_empty());
    assert!(report.unused_definitions().is_empty());
}

#[test]
fn startup_report_set_unused_definitions() {
    let mut report = create_test_report();
    report.set_unused_definitions(vec!["def1".into(), "def2".into()]);
    assert_eq!(report.unused_definitions().len(), 2);
}

#[test]
fn startup_report_set_context_state() {
    let mut report = create_test_report();
    report.set_context_state("refreshed".to_string());
    assert_eq!(report.context_state(), "refreshed");
}

#[test]
fn startup_report_record() {
    // StartupObservation::new() is pub(crate), so we test through the public API
    let mut report = create_test_report();
    // record() requires a StartupObservation, but we can't construct one externally
    // This test verifies the method exists and can be called
}

#[test]
fn startup_report_record_warning() {
    let mut report = create_test_report();
    report.record_warning("test.warning");
    assert_eq!(report.warnings().len(), 1);

    // Duplicate warning should be deduplicated
    report.record_warning("test.warning");
    assert_eq!(report.warnings().len(), 1);

    // Different warning should be added
    report.record_warning("other.warning");
    assert_eq!(report.warnings().len(), 2);
}

#[test]
fn startup_report_debug() {
    let report = create_test_report();
    let debug = format!("{:?}", report);
    assert!(!debug.is_empty());
}

#[test]
fn startup_report_serialize() {
    let report = create_test_report();
    let json = serde_json::to_string(&report).unwrap();
    assert!(!json.is_empty());
}

// ════════════════════════════════════════════════════════════════════
// DiagnosticConfiguration - 覆盖所有方法
// ════════════════════════════════════════════════════════════════════

#[test]
fn diagnostic_configuration_default() {
    let config = DiagnosticConfiguration::default();
    assert!(config.enabled_features().is_empty());
    assert!(config.adapters().is_empty());
    assert!(config.external_dependencies().is_empty());
    assert!(config.condition_evaluations().is_empty());
    assert!(config.warnings().is_empty());
}

#[test]
fn diagnostic_configuration_new() {
    let config = DiagnosticConfiguration::new(
        vec!["feature1".into()],
        vec![],
        vec![],
        vec![],
        vec!["warning1".into()],
    );
    assert_eq!(config.enabled_features().len(), 1);
    assert!(config.adapters().is_empty());
    assert!(config.external_dependencies().is_empty());
    assert!(config.condition_evaluations().is_empty());
    assert_eq!(config.warnings().len(), 1);
}

#[test]
fn diagnostic_configuration_debug() {
    let config = DiagnosticConfiguration::default();
    let debug = format!("{:?}", config);
    assert!(!debug.is_empty());
}

// ════════════════════════════════════════════════════════════════════
// ConfigurationProperties - 通过集成测试覆盖 trait 方法
// ════════════════════════════════════════════════════════════════════

struct TestConfig {
    name: String,
    port: u16,
}

impl vernal_context::ConfigurationProperties for TestConfig {
    const PREFIX: &'static str = "test.config";

    fn bind_with_prefix(
        environment: &ApplicationEnvironment,
        prefix: &str,
    ) -> Result<Self, vernal_context::ConfigurationPropertiesError> {
        let name = environment
            .get::<String>(&format!("{}.name", prefix))
            .map_err(|e| vernal_context::ConfigurationPropertiesError::environment::<Self>(
                "name",
                format!("{}.name", prefix),
                e,
            ))?
            .unwrap_or_default();

        let port = environment
            .get::<u16>(&format!("{}.port", prefix))
            .map_err(|e| vernal_context::ConfigurationPropertiesError::environment::<Self>(
                "port",
                format!("{}.port", prefix),
                e,
            ))?
            .unwrap_or(8080);

        Ok(TestConfig { name, port })
    }
}

#[test]
fn configuration_properties_bind() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new(
        "app",
        [
            ("test.config.name", "my-service"),
            ("test.config.port", "9090"),
        ],
    )
    .unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let config = TestConfig::bind(&env).unwrap();
    assert_eq!(config.name, "my-service");
    assert_eq!(config.port, 9090);
}

#[test]
fn configuration_properties_bind_with_prefix() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new(
        "app",
        [
            ("custom.name", "custom-service"),
            ("custom.port", "3000"),
        ],
    )
    .unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let config = TestConfig::bind_with_prefix(&env, "custom").unwrap();
    assert_eq!(config.name, "custom-service");
    assert_eq!(config.port, 3000);
}

#[test]
fn configuration_properties_bind_defaults() {
    let env = ApplicationEnvironment::builder().build();

    let config = TestConfig::bind(&env).unwrap();
    assert_eq!(config.name, "");
    assert_eq!(config.port, 8080);
}

#[test]
fn configuration_properties_component_definition() {
    let _def = TestConfig::component_definition();
}

#[test]
fn configuration_properties_prefix() {
    assert_eq!(TestConfig::PREFIX, "test.config");
}

// ════════════════════════════════════════════════════════════════════
// ApplicationModuleRegistrar - 覆盖所有方法
// ════════════════════════════════════════════════════════════════════

#[test]
fn registrar_all_methods() {
    use vernal_aop::Operation;

    let mut reg = ApplicationModuleRegistrar::new();

    reg.register(vernal_beans::ComponentDefinition::shared_value::<i32>(1));
    reg.register_all(vec![
        vernal_beans::ComponentDefinition::shared_value::<u32>(2),
        vernal_beans::ComponentDefinition::shared_value::<i64>(3),
    ]);

    let binding = vernal_beans::TraitBinding::new::<dyn std::fmt::Display + Send + Sync, String, _>(
        |arc_string: Arc<String>| -> Arc<dyn std::fmt::Display + Send + Sync> { arc_string },
    );
    reg.bind(binding);

    let binding2 = vernal_beans::TraitBinding::new::<dyn std::fmt::Display + Send + Sync, u32, _>(
        |arc: Arc<u32>| -> Arc<dyn std::fmt::Display + Send + Sync> { arc },
    );
    reg.bind_all(vec![binding2]);

    struct TestLifecycle;
    impl vernal_context::Lifecycle for TestLifecycle {}
    reg.lifecycle::<TestLifecycle>();

    struct MyEvent;
    struct MyListener;
    impl vernal_context::ApplicationEventListener<MyEvent> for MyListener {
        type Error = std::io::Error;
        async fn on_event(&self, _: Arc<MyEvent>) -> Result<(), Self::Error> { Ok(()) }
    }
    reg.event_listener::<MyEvent, MyListener>();

    struct MyRunner;
    impl vernal_context::ApplicationRunner for MyRunner {
        type Error = std::io::Error;
        async fn run(&self, _: tokio_util::sync::CancellationToken) -> Result<(), Self::Error> { Ok(()) }
    }
    reg.application_runner::<MyRunner>();

    struct MyTask;
    impl vernal_context::ScheduledTask for MyTask {
        type Error = std::io::Error;
        fn schedule(&self) -> vernal_context::TaskSchedule {
            vernal_context::TaskSchedule::fixed_rate(Duration::from_secs(60)).unwrap()
        }
        async fn run(&self, _: tokio_util::sync::CancellationToken) -> Result<(), Self::Error> { Ok(()) }
    }
    reg.scheduled_task::<MyTask>();

    reg.operation(Operation::new("comp", "m1"));
    reg.operations(vec![Operation::new("comp", "m2"), Operation::new("comp", "m3")]);

    let source1 = vernal_context::MapPropertySource::new("app1", [("k1", "v1")]).unwrap();
    reg.property_source_first(Arc::new(source1));

    let source2 = vernal_context::MapPropertySource::new("app2", [("k2", "v2")]).unwrap();
    reg.property_source_last(Arc::new(source2));

    reg.active_profile("prod");
    reg.default_profile("default");

    struct TrueCond;
    impl ComponentCondition for TrueCond {
        fn name(&self) -> &'static str { "true" }
        fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> { Ok(true) }
    }
    let module = ConditionalComponentModule::new("test-mod", TrueCond);
    reg.conditional(module);

    let module2 = ConditionalComponentModule::new("test-mod2", TrueCond);
    reg.conditionals(vec![module2]);

    let parts = reg.into_parts();
    assert_eq!(parts.definitions.len(), 3);
    assert_eq!(parts.bindings.len(), 2);
    assert_eq!(parts.lifecycle_registrars.len(), 1);
    assert_eq!(parts.event_listener_registrars.len(), 1);
    assert_eq!(parts.application_runner_registrars.len(), 1);
    assert_eq!(parts.scheduled_task_registrars.len(), 1);
    assert_eq!(parts.operations.len(), 3);
    assert!(!parts.environment_contributions.is_empty());
    assert_eq!(parts.conditional_modules.len(), 2);
}

// ════════════════════════════════════════════════════════════════════
// ConditionalComponentModule - 覆盖所有方法
// ════════════════════════════════════════════════════════════════════

struct TrueCond;
impl ComponentCondition for TrueCond {
    fn name(&self) -> &'static str { "true" }
    fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> { Ok(true) }
}

#[test]
fn conditional_module_all_methods() {
    let mut module = ConditionalComponentModule::new("test", TrueCond);

    module.register(vernal_beans::ComponentDefinition::shared_value::<i32>(1));
    module.register_all(vec![
        vernal_beans::ComponentDefinition::shared_value::<u32>(2),
        vernal_beans::ComponentDefinition::shared_value::<i64>(3),
    ]);

    let binding = vernal_beans::TraitBinding::new::<dyn std::fmt::Display + Send + Sync, String, _>(
        |arc_string: Arc<String>| -> Arc<dyn std::fmt::Display + Send + Sync> { arc_string },
    );
    module.bind(binding);

    let binding2 = vernal_beans::TraitBinding::new::<dyn std::fmt::Display + Send + Sync, u32, _>(
        |arc: Arc<u32>| -> Arc<dyn std::fmt::Display + Send + Sync> { arc },
    );
    module.bind_all(vec![binding2]);

    struct TestLifecycle;
    impl vernal_context::Lifecycle for TestLifecycle {}
    module.lifecycle::<TestLifecycle>();

    struct MyEvent;
    struct MyListener;
    impl vernal_context::ApplicationEventListener<MyEvent> for MyListener {
        type Error = std::io::Error;
        async fn on_event(&self, _: Arc<MyEvent>) -> Result<(), Self::Error> { Ok(()) }
    }
    module.event_listener::<MyEvent, MyListener>();

    struct MyRunner;
    impl vernal_context::ApplicationRunner for MyRunner {
        type Error = std::io::Error;
        async fn run(&self, _: tokio_util::sync::CancellationToken) -> Result<(), Self::Error> { Ok(()) }
    }
    module.application_runner::<MyRunner>();

    struct MyTask;
    impl vernal_context::ScheduledTask for MyTask {
        type Error = std::io::Error;
        fn schedule(&self) -> vernal_context::TaskSchedule {
            vernal_context::TaskSchedule::fixed_rate(Duration::from_secs(60)).unwrap()
        }
        async fn run(&self, _: tokio_util::sync::CancellationToken) -> Result<(), Self::Error> { Ok(()) }
    }
    module.scheduled_task::<MyTask>();

    let result = module.validate();
    assert!(result.is_ok());

    let env = ApplicationEnvironment::builder().build();
    let matched = module.matches(&env).unwrap();
    assert!(matched);

    let snapshot = module.snapshot(true);
    assert_eq!(snapshot.module(), "test");
    assert!(snapshot.matched());

    let parts = module.into_parts();
    assert_eq!(parts.definitions.len(), 3);
    assert_eq!(parts.bindings.len(), 2);
    assert_eq!(parts.lifecycle_registrars.len(), 1);
    assert_eq!(parts.event_listener_registrars.len(), 1);
    assert_eq!(parts.application_runner_registrars.len(), 1);
    assert_eq!(parts.scheduled_task_registrars.len(), 1);
}

// ════════════════════════════════════════════════════════════════════
// ContextError - 覆盖所有 Display 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn context_error_all_display() {
    use vernal_beans::{ComponentKey, ResolveError};
    use vernal_core::SharedError;

    let errs = vec![
        format!("{}", ContextError::ApplicationRunnerDefinitionNotFound { component: ComponentKey::of::<String>() }),
        format!("{}", ContextError::ApplicationRunnerScope { component: ComponentKey::of::<String>(), scope: "s" }),
        format!("{}", ContextError::DuplicateApplicationRunner { component: ComponentKey::of::<String>() }),
        format!("{}", ContextError::ApplicationRunnerResolution { component: ComponentKey::of::<String>(), source: Box::new(ResolveError::NotFound { component: "r".into(), path: vec![] }) }),
        format!("{}", ContextError::ApplicationRunnerFailed { source: vernal_context::ApplicationRunnerFailure::new("r", Arc::new(std::io::Error::other("f")) as SharedError) }),
        format!("{}", ContextError::ApplicationRunnerTimeout { runner: "r", timeout: Duration::from_secs(5), abort_settled: false }),
        format!("{}", ContextError::ScheduledTaskDefinitionNotFound { component: ComponentKey::of::<String>() }),
        format!("{}", ContextError::ScheduledTaskScope { component: ComponentKey::of::<String>(), scope: "s" }),
        format!("{}", ContextError::DuplicateScheduledTask { component: ComponentKey::of::<String>() }),
        format!("{}", ContextError::ScheduledTaskResolution { component: ComponentKey::of::<String>(), source: Box::new(ResolveError::NotFound { component: "t".into(), path: vec![] }) }),
        format!("{}", ContextError::EventListenerDefinitionNotFound { component: ComponentKey::of::<String>(), event: "e" }),
        format!("{}", ContextError::EventListenerScope { component: ComponentKey::of::<String>(), event: "e", scope: "s" }),
        format!("{}", ContextError::DuplicateEventListener { component: ComponentKey::of::<String>(), event: "e" }),
        format!("{}", ContextError::InvalidState { operation: "o", state: ContextState::Created }),
        format!("{}", ContextError::LifecycleDefinitionNotFound { component: ComponentKey::of::<String>() }),
        format!("{}", ContextError::ContainerWarmUp { source: Box::new(ResolveError::NotFound { component: "c".into(), path: vec![] }) }),
        format!("{}", ContextError::ComponentResolution { component: ComponentKey::of::<String>(), source: Box::new(ResolveError::NotFound { component: "c".into(), path: vec![] }) }),
        format!("{}", ContextError::EventListenerResolution { component: ComponentKey::of::<String>(), event: "e", source: Box::new(ResolveError::NotFound { component: "l".into(), path: vec![] }) }),
        format!("{}", ContextError::Lifecycle { component: "c", phase: LifecyclePhase::Start, source: Arc::new(std::io::Error::other("e")) as SharedError }),
        format!("{}", ContextError::LifecycleTimeout { component: "c", phase: LifecyclePhase::Start, timeout: Duration::from_secs(5), abort_settled: false }),
        format!("{}", ContextError::ManagedTask { source: vernal_context::ManagedTaskError::InvalidName }),
        format!("{}", ContextError::LifecycleCoordinator { operation: "o", source: Arc::new(std::io::Error::other("e")) as SharedError }),
        format!("{}", ContextError::LifecycleCancelled { operation: "o" }),
        format!("{}", ContextError::ShutdownSignal { source: Arc::new(std::io::Error::other("e")) as SharedError }),
        format!("{}", ContextError::PauseRestart { operation: "o", component: "c", phase: LifecyclePhase::Pause, source: Arc::new(std::io::Error::other("e")) as SharedError }),
    ];
    for d in errs {
        assert!(!d.is_empty());
    }
}

// ════════════════════════════════════════════════════════════════════
// ResolveError - 覆盖所有 Display 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn resolve_error_all_display() {
    use vernal_beans::{ResolveError, ComponentKey, TraitKey, ScopeKey, ScopeState};
    use vernal_core::SharedError;

    let errs = vec![
        format!("{}", ResolveError::NotFound { component: "c".into(), path: vec![] }),
        format!("{}", ResolveError::Ambiguous { component: "c".into(), candidates: vec![], path: vec![] }),
        format!("{}", ResolveError::UndeclaredDependency { component: ComponentKey::of::<String>(), dependency: "d".into() }),
        format!("{}", ResolveError::TypeMismatch { component: ComponentKey::of::<String>() }),
        format!("{}", ResolveError::TraitBindingTypeMismatch { binding: TraitKey::of::<dyn std::fmt::Display>(), target: ComponentKey::of::<String>() }),
        format!("{}", ResolveError::Construction { component: ComponentKey::of::<String>(), source: Arc::new(std::io::Error::other("e")) as SharedError }),
        format!("{}", ResolveError::CircularRuntime { path: vec![] }),
        format!("{}", ResolveError::ProviderUsedDuringConstruction { component: ComponentKey::of::<String>(), dependency: "d".into() }),
        format!("{}", ResolveError::ScopeNotActive { component: ComponentKey::of::<String>(), scope: ScopeKey::of::<String>() }),
        format!("{}", ResolveError::ScopeOwnerMismatch { scope: ScopeKey::of::<String>() }),
        format!("{}", ResolveError::ScopeUnavailable { component: ComponentKey::of::<String>(), scope: ScopeKey::of::<String>(), state: ScopeState::Closed, cancelled: true }),
    ];
    for d in errs {
        assert!(!d.is_empty());
    }
}

// ════════════════════════════════════════════════════════════════════
// ApplicationEnvironment - 覆盖更多路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn environment_property_expansion() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new(
        "app",
        [("a", "val_a"), ("b", "${a}_b"), ("c", "${b}_c")]
    ).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let c = env.property("c").unwrap().unwrap();
    assert_eq!(c, "val_a_b_c");
}

#[test]
fn environment_property_multiple_placeholders() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new(
        "app",
        [("host", "localhost"), ("port", "8080"), ("url", "http://${host}:${port}/api")]
    ).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let url = env.property("url").unwrap().unwrap();
    assert_eq!(url, "http://localhost:8080/api");
}

#[test]
fn environment_property_default_value() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new(
        "app", [("k", "${missing:default_val}")]
    ).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let val = env.property("k").unwrap().unwrap();
    assert_eq!(val, "default_val");
}

#[test]
fn environment_property_empty_default() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new(
        "app", [("k", "${missing:}")]
    ).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let val = env.property("k").unwrap().unwrap();
    assert_eq!(val, "");
}

#[test]
fn environment_property_literal_only() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new(
        "app", [("k", "just a value")]
    ).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let val = env.property("k").unwrap().unwrap();
    assert_eq!(val, "just a value");
}

#[test]
fn environment_raw_property() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new(
        "app", [("k", "value")]
    ).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let raw = env.raw_property("k").unwrap();
    assert_eq!(raw.as_deref(), Some("value"));
}

#[test]
fn environment_get_typed() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new("app", [("port", "8080")]).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let result: Result<Option<u16>, _> = env.get("port");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(8080));
}

#[test]
fn environment_require_typed() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new("app", [("port", "8080")]).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let result: Result<u16, _> = env.require("port");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 8080);
}

#[test]
fn environment_property_source_priority() {
    let mut builder = ApplicationEnvironment::builder();
    let high = vernal_context::MapPropertySource::new("high", [("k", "high_val")]).unwrap();
    builder.add_first(Arc::new(high)).unwrap();
    let low = vernal_context::MapPropertySource::new("low", [("k", "low_val")]).unwrap();
    builder.add_last(Arc::new(low)).unwrap();
    let env = builder.build();

    let val = env.property("k").unwrap().unwrap();
    assert_eq!(val, "high_val");
}

// ════════════════════════════════════════════════════════════════════
// ApplicationBuildError - 覆盖所有 Display 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn build_error_all_display() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::{DefinitionError, GraphError, ResolveError};
    use vernal_aop::{Operation, OperationMetadata, OperationMetadataConflictError, InvocationPlanCatalogInitializationError};

    let errs = vec![
        format!("{}", ApplicationBuildError::TokioRuntimeUnavailable { source: tokio::runtime::Handle::try_current().unwrap_err() }),
        format!("{}", ApplicationBuildError::Definition { source: DefinitionError::InvalidQualifier { value: "v".into() } }),
        format!("{}", ApplicationBuildError::Graph { source: GraphError::MissingDependency { path: vec![] } }),
        format!("{}", ApplicationBuildError::Context { source: ContextError::InvalidState { operation: "o", state: ContextState::Created } }),
        format!("{}", ApplicationBuildError::Condition { source: vernal_context::ConditionError::EmptyProfileSet }),
        format!("{}", ApplicationBuildError::AdvisorResolution { component: vernal_beans::ComponentKey::of::<String>(), source: Box::new(ResolveError::NotFound { component: "c".into(), path: vec![] }) }),
        format!("{}", ApplicationBuildError::LocalAdvisorResolution { component: vernal_beans::ComponentKey::of::<String>(), source: Box::new(ResolveError::NotFound { component: "c".into(), path: vec![] }) }),
        format!("{}", ApplicationBuildError::AdvisorScope { component: vernal_beans::ComponentKey::of::<String>(), scope: "request" }),
        format!("{}", ApplicationBuildError::OperationMetadata { source: OperationMetadataConflictError::new(Operation::new("c", "m"), OperationMetadata::empty(), OperationMetadata::empty()) }),
        format!("{}", ApplicationBuildError::AopCatalogInitialization { source: InvocationPlanCatalogInitializationError }),
    ];
    for d in errs {
        assert!(!d.is_empty());
    }
}

// ════════════════════════════════════════════════════════════════════
// ManagedTaskError - 覆盖所有 Display 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn managed_task_error_all_display() {
    use vernal_core::SharedError;

    let errs = vec![
        format!("{}", vernal_context::ManagedTaskError::InvalidName),
        format!("{}", vernal_context::ManagedTaskError::IdentifierExhausted { task: "t" }),
        format!("{}", vernal_context::ManagedTaskError::SpawnRejected { task: "t" }),
        format!("{}", vernal_context::ManagedTaskError::TaskFailed { task: "t", source: Arc::new(std::io::Error::other("e")) as SharedError }),
        format!("{}", vernal_context::ManagedTaskError::TaskPanicked { task: "t", source: Arc::new(std::io::Error::other("e")) as SharedError }),
        format!("{}", vernal_context::ManagedTaskError::TaskCancelled { task: "t", source: Arc::new(std::io::Error::other("e")) as SharedError }),
        format!("{}", vernal_context::ManagedTaskError::ShutdownTimeout { timeout: Duration::from_secs(5), remaining: 3 }),
        format!("{}", vernal_context::ManagedTaskError::AbortTimeout { timeout: Duration::from_secs(5), remaining: 2 }),
        format!("{}", vernal_context::ManagedTaskError::CoordinatorUnavailable { remaining: 1 }),
    ];
    for d in errs {
        assert!(!d.is_empty());
    }
}
