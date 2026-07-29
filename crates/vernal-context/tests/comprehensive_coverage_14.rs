//! 全面覆盖测试 - Part 14: 通过实现 Component + Interceptor/LocalInterceptor
//! 测试 register_advisor_component 和 register_local_advisor_component 方法

use std::sync::Arc;
use std::time::Duration;

use vernal_context::{
    ApplicationEnvironment, ApplicationModuleRegistrar,
    ApplicationLaunchError, StartupReport, DiagnosticConfiguration,
    ConditionalComponentModule, ComponentCondition,
    ConfigurationProperties, ConfigurationPropertiesError,
    ContextError, ContextState, LifecyclePhase,
};

// ════════════════════════════════════════════════════════════════════
// 实现 Component + Interceptor 用于测试 register_advisor_component
// ════════════════════════════════════════════════════════════════════

struct TestAdvisorComponent;

impl vernal_aop::Interceptor for TestAdvisorComponent {
    fn intercept<'a>(
        &'a self,
        _invocation: Arc<vernal_aop::Invocation>,
        next: vernal_aop::Next<'a>,
    ) -> vernal_aop::InvocationFuture<'a> {
        next.run(_invocation)
    }
}

impl vernal_beans::Component for TestAdvisorComponent {
    fn definition() -> vernal_beans::ComponentDefinition {
        vernal_beans::ComponentDefinition::shared_value::<i32>(42)
    }
}

struct TestPointcut;

impl vernal_aop::Pointcut for TestPointcut {
    fn matches(&self, _operation: &vernal_aop::Operation) -> bool {
        true
    }
}

// ════════════════════════════════════════════════════════════════════
// 实现 Component + LocalInterceptor 用于测试 register_local_advisor_component
// ════════════════════════════════════════════════════════════════════

struct TestLocalAdvisorComponent;

impl vernal_aop::LocalInterceptor for TestLocalAdvisorComponent {
    fn intercept_local<'a>(
        &'a self,
        _invocation: Arc<vernal_aop::Invocation>,
        next: vernal_aop::LocalNext<'a>,
    ) -> vernal_aop::LocalInvocationFuture<'a> {
        next.run(_invocation)
    }
}

impl vernal_beans::Component for TestLocalAdvisorComponent {
    fn definition() -> vernal_beans::ComponentDefinition {
        vernal_beans::ComponentDefinition::shared_value::<i32>(43)
    }
}

// ════════════════════════════════════════════════════════════════════
// ApplicationModuleRegistrar - 测试 register_advisor_component 方法
// ════════════════════════════════════════════════════════════════════

#[test]
fn registrar_register_advisor_component() {
    let mut reg = ApplicationModuleRegistrar::new();
    reg.register_advisor_component::<TestAdvisorComponent, TestPointcut>(TestPointcut, 10);

    let parts = reg.into_parts();
    assert_eq!(parts.definitions.len(), 1); // Component definition
    assert_eq!(parts.advisor_registrations.len(), 1); // Advisor registration
}

#[test]
fn registrar_register_local_advisor_component() {
    let mut reg = ApplicationModuleRegistrar::new();
    reg.register_local_advisor_component::<TestLocalAdvisorComponent, TestPointcut>(TestPointcut, 10);

    let parts = reg.into_parts();
    assert_eq!(parts.definitions.len(), 1); // Component definition
    assert_eq!(parts.local_advisor_registrations.len(), 1); // Local advisor registration
}

// ════════════════════════════════════════════════════════════════════
// ConditionalComponentModule - 测试所有 qualified 方法
// ════════════════════════════════════════════════════════════════════

struct TrueCond;
impl ComponentCondition for TrueCond {
    fn name(&self) -> &'static str { "true" }
    fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> { Ok(true) }
}

#[test]
fn conditional_module_all_qualified_methods() {
    use vernal_beans::Qualifier;

    let mut module = ConditionalComponentModule::new("test", TrueCond);

    // lifecycle_qualified
    struct TestLifecycle;
    impl vernal_context::Lifecycle for TestLifecycle {}
    let qualifier1 = Qualifier::new("lifecycle-qual").unwrap();
    module.lifecycle_qualified::<TestLifecycle>(qualifier1);

    // event_listener_qualified
    struct MyEvent;
    struct MyListener;
    impl vernal_context::ApplicationEventListener<MyEvent> for MyListener {
        type Error = std::io::Error;
        async fn on_event(&self, _: Arc<MyEvent>) -> Result<(), Self::Error> { Ok(()) }
    }
    let qualifier2 = Qualifier::new("listener-qual").unwrap();
    module.event_listener_qualified::<MyEvent, MyListener>(qualifier2);

    // application_runner_qualified
    struct MyRunner;
    impl vernal_context::ApplicationRunner for MyRunner {
        type Error = std::io::Error;
        async fn run(&self, _: tokio_util::sync::CancellationToken) -> Result<(), Self::Error> { Ok(()) }
    }
    let qualifier3 = Qualifier::new("runner-qual").unwrap();
    module.application_runner_qualified::<MyRunner>(qualifier3);

    // scheduled_task_qualified
    struct MyTask;
    impl vernal_context::ScheduledTask for MyTask {
        type Error = std::io::Error;
        fn schedule(&self) -> vernal_context::TaskSchedule {
            vernal_context::TaskSchedule::fixed_rate(Duration::from_secs(60)).unwrap()
        }
        async fn run(&self, _: tokio_util::sync::CancellationToken) -> Result<(), Self::Error> { Ok(()) }
    }
    let qualifier4 = Qualifier::new("task-qual").unwrap();
    module.scheduled_task_qualified::<MyTask>(qualifier4);

    // validate
    let result = module.validate();
    assert!(result.is_ok());

    // matches
    let env = ApplicationEnvironment::builder().build();
    let matched = module.matches(&env).unwrap();
    assert!(matched);

    // snapshot
    let snapshot = module.snapshot(true);
    assert_eq!(snapshot.module(), "test");
    assert!(snapshot.matched());

    // into_parts
    let parts = module.into_parts();
    assert_eq!(parts.lifecycle_registrars.len(), 1);
    assert_eq!(parts.event_listener_registrars.len(), 1);
    assert_eq!(parts.application_runner_registrars.len(), 1);
    assert_eq!(parts.scheduled_task_registrars.len(), 1);
}

// ════════════════════════════════════════════════════════════════════
// StartupReport - 覆盖所有 getter 方法
// ════════════════════════════════════════════════════════════════════

fn create_test_report() -> StartupReport {
    let env = ApplicationEnvironment::builder().build();
    let registry = vernal_beans::RegistrySnapshot::new(
        vernal_beans::RegistrySummary::new(5, 3, 2, 1, 10, 4),
        vec![],
        vec![],
    );
    let invocation_plans = vernal_aop::InvocationPlanCatalog::default();
    let local_invocation_plans = vernal_aop::LocalInvocationPlanCatalog::default();
    let diagnostics = DiagnosticConfiguration::new(
        vec!["feature1".into(), "feature2".into()],
        vec![],
        vec![],
        vec![],
        vec!["warning1".into()],
    );

    StartupReport::new(
        "refreshed".to_string(),
        &env,
        registry,
        &invocation_plans,
        &local_invocation_plans,
        &diagnostics,
    )
}

#[test]
fn startup_report_all_getters() {
    let report = create_test_report();

    // String getters
    assert!(!report.framework_version().is_empty());
    assert!(!report.minimum_rust_version().is_empty());
    assert!(!report.project_status().is_empty());
    assert_eq!(report.context_state(), "refreshed");

    // Reference getters
    let _env = report.environment();
    let _registry = report.registry();

    // Count getters
    assert_eq!(report.aop_plan_count(), 0);
    assert_eq!(report.aop_interceptor_count(), 0);
    assert_eq!(report.local_aop_plan_count(), 0);
    assert_eq!(report.local_aop_interceptor_count(), 0);

    // Slice getters
    assert_eq!(report.enabled_features().len(), 2);
    assert!(report.adapters().is_empty());
    assert!(report.external_dependencies().is_empty());
    assert!(report.condition_evaluations().is_empty());
    assert!(report.observations().is_empty());
    assert_eq!(report.warnings().len(), 1);
    assert!(report.unused_definitions().is_empty());
}

#[test]
fn startup_report_setters() {
    let mut report = create_test_report();

    // set_unused_definitions
    report.set_unused_definitions(vec!["def1".into(), "def2".into()]);
    assert_eq!(report.unused_definitions().len(), 2);

    // set_context_state
    report.set_context_state("starting".to_string());
    assert_eq!(report.context_state(), "starting");
}

#[test]
fn startup_report_record_warning() {
    let mut report = create_test_report();

    // Add warnings
    report.record_warning("warning2");
    report.record_warning("warning3");

    // Check deduplication
    report.record_warning("warning2");
    assert_eq!(report.warnings().len(), 3); // warning1 + warning2 + warning3
}

#[test]
fn startup_report_debug() {
    let report = create_test_report();
    let debug = format!("{:?}", report);
    assert!(!debug.is_empty());
    assert!(debug.contains("StartupReport"));
}

#[test]
fn startup_report_serialize() {
    let report = create_test_report();
    let json = serde_json::to_string(&report).unwrap();
    assert!(!json.is_empty());
    assert!(json.contains("framework_version"));
    assert!(json.contains("context_state"));
}

// ════════════════════════════════════════════════════════════════════
// ConfigurationProperties - 通过集成测试覆盖 trait 方法
// ════════════════════════════════════════════════════════════════════

struct TestConfig {
    name: String,
    port: u16,
    debug: bool,
}

impl ConfigurationProperties for TestConfig {
    const PREFIX: &'static str = "test.config";

    fn bind_with_prefix(
        environment: &ApplicationEnvironment,
        prefix: &str,
    ) -> Result<Self, ConfigurationPropertiesError> {
        let name = environment
            .get::<String>(&format!("{}.name", prefix))
            .map_err(|e| ConfigurationPropertiesError::environment::<Self>(
                "name",
                format!("{}.name", prefix),
                e,
            ))?
            .unwrap_or_default();

        let port = environment
            .get::<u16>(&format!("{}.port", prefix))
            .map_err(|e| ConfigurationPropertiesError::environment::<Self>(
                "port",
                format!("{}.port", prefix),
                e,
            ))?
            .unwrap_or(8080);

        let debug = environment
            .get::<bool>(&format!("{}.debug", prefix))
            .map_err(|e| ConfigurationPropertiesError::environment::<Self>(
                "debug",
                format!("{}.debug", prefix),
                e,
            ))?
            .unwrap_or(false);

        Ok(TestConfig { name, port, debug })
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
            ("test.config.debug", "true"),
        ],
    )
    .unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let config = TestConfig::bind(&env).unwrap();
    assert_eq!(config.name, "my-service");
    assert_eq!(config.port, 9090);
    assert!(config.debug);
}

#[test]
fn configuration_properties_bind_with_prefix() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new(
        "app",
        [
            ("custom.name", "custom-service"),
            ("custom.port", "3000"),
            ("custom.debug", "false"),
        ],
    )
    .unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let config = TestConfig::bind_with_prefix(&env, "custom").unwrap();
    assert_eq!(config.name, "custom-service");
    assert_eq!(config.port, 3000);
    assert!(!config.debug);
}

#[test]
fn configuration_properties_bind_defaults() {
    let env = ApplicationEnvironment::builder().build();

    let config = TestConfig::bind(&env).unwrap();
    assert_eq!(config.name, "");
    assert_eq!(config.port, 8080);
    assert!(!config.debug);
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
// ApplicationLaunchError - 覆盖所有 Display 分支
// ════════════════════════════════════════════════════════════════════

#[test]
fn launch_error_build_display() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::DefinitionError;

    let build_err = ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier { value: "v".into() },
    };
    let launch_err = ApplicationLaunchError::build(build_err);
    let display = format!("{launch_err}");
    assert!(display.contains("application build failed"));
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
fn launch_error_lifecycle_with_cleanup_display() {
    let report = create_test_report();
    let ctx_err = ContextError::LifecycleCancelled { operation: "refresh" };
    let cleanup = ContextError::LifecycleCancelled { operation: "cleanup" };
    let launch_err = ApplicationLaunchError::lifecycle("refresh", ctx_err, Some(cleanup), report);
    let display = format!("{launch_err}");
    assert!(display.contains("context cleanup also failed"));
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
fn launch_error_coordinator_with_cleanup_display() {
    let report = create_test_report();
    let source = std::io::Error::other("coordinator failed");
    let cleanup = ContextError::LifecycleCancelled { operation: "cleanup" };
    let launch_err = ApplicationLaunchError::coordinator(source, Some(cleanup), report);
    let display = format!("{launch_err}");
    assert!(display.contains("context cleanup also failed"));
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
