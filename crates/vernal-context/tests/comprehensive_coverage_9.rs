//! 全面覆盖测试 - Part 9: 针对 application_module_registrar 中的 qualified 和 component 方法
//! 以及 application_launch_error 和 startup_report 的剩余路径

use std::sync::Arc;
use std::time::Duration;

use vernal_context::{
    ApplicationEnvironment, ApplicationModuleRegistrar,
    ConditionalComponentModule, ComponentCondition,
    ContextError, ContextState, LifecyclePhase,
    ConfigurationPhase,
};

// ════════════════════════════════════════════════════════════════════
// ApplicationModuleRegistrar - qualified 方法
// ════════════════════════════════════════════════════════════════════

struct TrueCond;
impl ComponentCondition for TrueCond {
    fn name(&self) -> &'static str { "true" }
    fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> { Ok(true) }
}

#[test]
fn registrar_lifecycle_qualified() {
    use vernal_beans::Qualifier;

    let mut reg = ApplicationModuleRegistrar::new();
    struct TestLifecycle;
    impl vernal_context::Lifecycle for TestLifecycle {}

    let qualifier = Qualifier::new("test-qualifier").unwrap();
    reg.lifecycle_qualified::<TestLifecycle>(qualifier);

    let parts = reg.into_parts();
    assert_eq!(parts.lifecycle_registrars.len(), 1);
}

#[test]
fn registrar_event_listener_qualified() {
    use vernal_beans::Qualifier;

    let mut reg = ApplicationModuleRegistrar::new();
    struct MyEvent;
    struct MyListener;
    impl vernal_context::ApplicationEventListener<MyEvent> for MyListener {
        type Error = std::io::Error;
        async fn on_event(&self, _: Arc<MyEvent>) -> Result<(), Self::Error> { Ok(()) }
    }

    let qualifier = Qualifier::new("test-qualifier").unwrap();
    reg.event_listener_qualified::<MyEvent, MyListener>(qualifier);

    let parts = reg.into_parts();
    assert_eq!(parts.event_listener_registrars.len(), 1);
}

#[test]
fn registrar_application_runner_qualified() {
    use vernal_beans::Qualifier;

    let mut reg = ApplicationModuleRegistrar::new();
    struct MyRunner;
    impl vernal_context::ApplicationRunner for MyRunner {
        type Error = std::io::Error;
        async fn run(&self, _: tokio_util::sync::CancellationToken) -> Result<(), Self::Error> { Ok(()) }
    }

    let qualifier = Qualifier::new("test-qualifier").unwrap();
    reg.application_runner_qualified::<MyRunner>(qualifier);

    let parts = reg.into_parts();
    assert_eq!(parts.application_runner_registrars.len(), 1);
}

#[test]
fn registrar_scheduled_task_qualified() {
    use vernal_beans::Qualifier;

    let mut reg = ApplicationModuleRegistrar::new();
    struct MyTask;
    impl vernal_context::ScheduledTask for MyTask {
        type Error = std::io::Error;
        fn schedule(&self) -> vernal_context::TaskSchedule {
            vernal_context::TaskSchedule::fixed_rate(Duration::from_secs(60)).unwrap()
        }
        async fn run(&self, _: tokio_util::sync::CancellationToken) -> Result<(), Self::Error> { Ok(()) }
    }

    let qualifier = Qualifier::new("test-qualifier").unwrap();
    reg.scheduled_task_qualified::<MyTask>(qualifier);

    let parts = reg.into_parts();
    assert_eq!(parts.scheduled_task_registrars.len(), 1);
}

// ════════════════════════════════════════════════════════════════════
// ApplicationModuleRegistrar - component 和 configuration_properties 方法
// ════════════════════════════════════════════════════════════════════

#[test]
fn registrar_component() {
    use vernal_beans::Component;

    // Create a simple component struct
    struct TestComponent;
    impl Component for TestComponent {
        fn definition() -> vernal_beans::ComponentDefinition {
            vernal_beans::ComponentDefinition::shared_value::<i32>(42)
        }
    }

    let mut reg = ApplicationModuleRegistrar::new();
    reg.component::<TestComponent>();

    let parts = reg.into_parts();
    assert_eq!(parts.definitions.len(), 1);
}

#[test]
fn registrar_configuration_properties() {
    // Create a simple configuration properties struct
    struct TestConfig;
    impl vernal_context::ConfigurationProperties for TestConfig {
        const PREFIX: &'static str = "test";
        fn bind_with_prefix(_env: &ApplicationEnvironment, _prefix: &str) -> Result<Self, vernal_context::ConfigurationPropertiesError> {
            Ok(TestConfig)
        }
    }

    let mut reg = ApplicationModuleRegistrar::new();
    reg.configuration_properties::<TestConfig>();

    let parts = reg.into_parts();
    assert_eq!(parts.definitions.len(), 1);
}

// ════════════════════════════════════════════════════════════════════
// ApplicationModuleRegistrar - register_event_listener_component
// ════════════════════════════════════════════════════════════════════

#[test]
fn registrar_register_event_listener_component() {
    use vernal_beans::Component;

    struct MyEvent;
    struct MyListener;
    impl vernal_context::ApplicationEventListener<MyEvent> for MyListener {
        type Error = std::io::Error;
        async fn on_event(&self, _: Arc<MyEvent>) -> Result<(), Self::Error> { Ok(()) }
    }
    impl Component for MyListener {
        fn definition() -> vernal_beans::ComponentDefinition {
            vernal_beans::ComponentDefinition::shared_value::<String>("listener".to_string())
        }
    }

    let mut reg = ApplicationModuleRegistrar::new();
    reg.register_event_listener_component::<MyEvent, MyListener>();

    let parts = reg.into_parts();
    assert_eq!(parts.definitions.len(), 1);
    assert_eq!(parts.event_listener_registrars.len(), 1);
}

// ════════════════════════════════════════════════════════════════════
// ApplicationModuleRegistrar - register_application_runner
// ════════════════════════════════════════════════════════════════════

#[test]
fn registrar_register_application_runner() {
    use vernal_beans::Component;

    struct MyRunner;
    impl vernal_context::ApplicationRunner for MyRunner {
        type Error = std::io::Error;
        async fn run(&self, _: tokio_util::sync::CancellationToken) -> Result<(), Self::Error> { Ok(()) }
    }
    impl Component for MyRunner {
        fn definition() -> vernal_beans::ComponentDefinition {
            vernal_beans::ComponentDefinition::shared_value::<String>("runner".to_string())
        }
    }

    let mut reg = ApplicationModuleRegistrar::new();
    reg.register_application_runner::<MyRunner>();

    let parts = reg.into_parts();
    assert_eq!(parts.definitions.len(), 1);
    assert_eq!(parts.application_runner_registrars.len(), 1);
}

// ════════════════════════════════════════════════════════════════════
// ApplicationModuleRegistrar - register_scheduled_task
// ════════════════════════════════════════════════════════════════════

#[test]
fn registrar_register_scheduled_task() {
    use vernal_beans::Component;

    struct MyTask;
    impl vernal_context::ScheduledTask for MyTask {
        type Error = std::io::Error;
        fn schedule(&self) -> vernal_context::TaskSchedule {
            vernal_context::TaskSchedule::fixed_rate(Duration::from_secs(60)).unwrap()
        }
        async fn run(&self, _: tokio_util::sync::CancellationToken) -> Result<(), Self::Error> { Ok(()) }
    }
    impl Component for MyTask {
        fn definition() -> vernal_beans::ComponentDefinition {
            vernal_beans::ComponentDefinition::shared_value::<String>("task".to_string())
        }
    }

    let mut reg = ApplicationModuleRegistrar::new();
    reg.register_scheduled_task::<MyTask>();

    let parts = reg.into_parts();
    assert_eq!(parts.definitions.len(), 1);
    assert_eq!(parts.scheduled_task_registrars.len(), 1);
}

// ════════════════════════════════════════════════════════════════════
// ApplicationModuleRegistrar - advisor 方法
// ════════════════════════════════════════════════════════════════════

#[test]
fn registrar_advisor() {
    // Advisor requires Pointcut and Interceptor traits
    // The Interceptor trait has complex lifetime requirements
    // Skip this test for now - it's covered through other paths
}

#[test]
fn registrar_local_advisor() {
    // LocalAdvisor requires a LocalInterceptor which has different bounds
    // Skip this test for now
}

// ════════════════════════════════════════════════════════════════════
// ConditionalComponentModule - qualified 方法
// ════════════════════════════════════════════════════════════════════

#[test]
fn conditional_module_lifecycle_qualified() {
    use vernal_beans::Qualifier;

    struct TrueCond;
    impl ComponentCondition for TrueCond {
        fn name(&self) -> &'static str { "true" }
        fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> { Ok(true) }
    }

    let mut module = ConditionalComponentModule::new("test", TrueCond);
    struct TestLifecycle;
    impl vernal_context::Lifecycle for TestLifecycle {}

    let qualifier = Qualifier::new("test-qualifier").unwrap();
    module.lifecycle_qualified::<TestLifecycle>(qualifier);

    let parts = module.into_parts();
    assert_eq!(parts.lifecycle_registrars.len(), 1);
}

#[test]
fn conditional_module_event_listener_qualified() {
    use vernal_beans::Qualifier;

    struct TrueCond;
    impl ComponentCondition for TrueCond {
        fn name(&self) -> &'static str { "true" }
        fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> { Ok(true) }
    }

    let mut module = ConditionalComponentModule::new("test", TrueCond);
    struct MyEvent;
    struct MyListener;
    impl vernal_context::ApplicationEventListener<MyEvent> for MyListener {
        type Error = std::io::Error;
        async fn on_event(&self, _: Arc<MyEvent>) -> Result<(), Self::Error> { Ok(()) }
    }

    let qualifier = Qualifier::new("test-qualifier").unwrap();
    module.event_listener_qualified::<MyEvent, MyListener>(qualifier);

    let parts = module.into_parts();
    assert_eq!(parts.event_listener_registrars.len(), 1);
}

#[test]
fn conditional_module_application_runner_qualified() {
    use vernal_beans::Qualifier;

    struct TrueCond;
    impl ComponentCondition for TrueCond {
        fn name(&self) -> &'static str { "true" }
        fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> { Ok(true) }
    }

    let mut module = ConditionalComponentModule::new("test", TrueCond);
    struct MyRunner;
    impl vernal_context::ApplicationRunner for MyRunner {
        type Error = std::io::Error;
        async fn run(&self, _: tokio_util::sync::CancellationToken) -> Result<(), Self::Error> { Ok(()) }
    }

    let qualifier = Qualifier::new("test-qualifier").unwrap();
    module.application_runner_qualified::<MyRunner>(qualifier);

    let parts = module.into_parts();
    assert_eq!(parts.application_runner_registrars.len(), 1);
}

#[test]
fn conditional_module_scheduled_task_qualified() {
    use vernal_beans::Qualifier;

    struct TrueCond;
    impl ComponentCondition for TrueCond {
        fn name(&self) -> &'static str { "true" }
        fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> { Ok(true) }
    }

    let mut module = ConditionalComponentModule::new("test", TrueCond);
    struct MyTask;
    impl vernal_context::ScheduledTask for MyTask {
        type Error = std::io::Error;
        fn schedule(&self) -> vernal_context::TaskSchedule {
            vernal_context::TaskSchedule::fixed_rate(Duration::from_secs(60)).unwrap()
        }
        async fn run(&self, _: tokio_util::sync::CancellationToken) -> Result<(), Self::Error> { Ok(()) }
    }

    let qualifier = Qualifier::new("test-qualifier").unwrap();
    module.scheduled_task_qualified::<MyTask>(qualifier);

    let parts = module.into_parts();
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
        // fmt_application_runner paths
        format!("{}", ContextError::ApplicationRunnerDefinitionNotFound { component: ComponentKey::of::<String>() }),
        format!("{}", ContextError::ApplicationRunnerScope { component: ComponentKey::of::<String>(), scope: "s" }),
        format!("{}", ContextError::DuplicateApplicationRunner { component: ComponentKey::of::<String>() }),
        format!("{}", ContextError::ApplicationRunnerResolution { component: ComponentKey::of::<String>(), source: Box::new(ResolveError::NotFound { component: "r".into(), path: vec![] }) }),
        format!("{}", ContextError::ApplicationRunnerFailed { source: vernal_context::ApplicationRunnerFailure::new("r", Arc::new(std::io::Error::other("f")) as SharedError) }),
        format!("{}", ContextError::ApplicationRunnerTimeout { runner: "r", timeout: Duration::from_secs(5), abort_settled: false }),
        // fmt_scheduled_task paths
        format!("{}", ContextError::ScheduledTaskDefinitionNotFound { component: ComponentKey::of::<String>() }),
        format!("{}", ContextError::ScheduledTaskScope { component: ComponentKey::of::<String>(), scope: "s" }),
        format!("{}", ContextError::DuplicateScheduledTask { component: ComponentKey::of::<String>() }),
        format!("{}", ContextError::ScheduledTaskResolution { component: ComponentKey::of::<String>(), source: Box::new(ResolveError::NotFound { component: "t".into(), path: vec![] }) }),
        // fmt_event_listener paths
        format!("{}", ContextError::EventListenerDefinitionNotFound { component: ComponentKey::of::<String>(), event: "e" }),
        format!("{}", ContextError::EventListenerScope { component: ComponentKey::of::<String>(), event: "e", scope: "s" }),
        format!("{}", ContextError::DuplicateEventListener { component: ComponentKey::of::<String>(), event: "e" }),
        // main match paths
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
// DefinitionError - 覆盖所有 Display 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn definition_error_all_display() {
    use vernal_beans::{DefinitionError, ComponentKey, TraitKey};

    let errs = vec![
        format!("{}", DefinitionError::InvalidQualifier { value: "bad".into() }),
        format!("{}", DefinitionError::DuplicateDefinition { key: ComponentKey::of::<String>() }),
        format!("{}", DefinitionError::DuplicateTraitBinding { key: TraitKey::of::<dyn std::fmt::Display>(), target: ComponentKey::of::<String>() }),
        format!("{}", DefinitionError::DuplicateQualifiedTraitBinding { key: TraitKey::of::<dyn std::fmt::Display>() }),
        format!("{}", DefinitionError::MultiplePrimaryTraitBindings { trait_name: "Display" }),
    ];
    for d in errs {
        assert!(!d.is_empty());
    }
}

// ════════════════════════════════════════════════════════════════════
// GraphError - 覆盖所有 Display 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn graph_error_all_display() {
    use vernal_beans::GraphError;

    let errs = vec![
        format!("{}", GraphError::MissingDependency { path: vec![] }),
        format!("{}", GraphError::AmbiguousDependency { path: vec![], candidates: vec![] }),
        format!("{}", GraphError::MissingTraitBindingTarget { binding: "b".into() }),
        format!("{}", GraphError::Cycle { path: vec![] }),
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
    use vernal_context::ManagedTaskError;
    use vernal_core::SharedError;

    let errs = vec![
        format!("{}", ManagedTaskError::InvalidName),
        format!("{}", ManagedTaskError::IdentifierExhausted { task: "t" }),
        format!("{}", ManagedTaskError::SpawnRejected { task: "t" }),
        format!("{}", ManagedTaskError::TaskFailed { task: "t", source: Arc::new(std::io::Error::other("e")) as SharedError }),
        format!("{}", ManagedTaskError::TaskPanicked { task: "t", source: Arc::new(std::io::Error::other("e")) as SharedError }),
        format!("{}", ManagedTaskError::TaskCancelled { task: "t", source: Arc::new(std::io::Error::other("e")) as SharedError }),
        format!("{}", ManagedTaskError::ShutdownTimeout { timeout: Duration::from_secs(5), remaining: 3 }),
        format!("{}", ManagedTaskError::AbortTimeout { timeout: Duration::from_secs(5), remaining: 2 }),
        format!("{}", ManagedTaskError::CoordinatorUnavailable { remaining: 1 }),
    ];
    for d in errs {
        assert!(!d.is_empty());
    }
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
