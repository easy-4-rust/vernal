//! 全面覆盖测试 - Part 4: 针对 application_build_error, application_module_registrar,
//! conditional_component_module 等文件的未覆盖路径。

use std::sync::Arc;
use std::time::Duration;

use vernal_context::{
    ApplicationEnvironment, ApplicationModuleRegistrar,
    ConditionalComponentModule, ComponentCondition,
    ContextError, ContextState, LifecyclePhase,
    ConfigurationPhase, ManagedTaskError,
};

// ════════════════════════════════════════════════════════════════════
// ApplicationBuildError - 覆盖所有10个 Display 分支和 Error::source()
// ════════════════════════════════════════════════════════════════════

#[test]
fn build_error_tokio_runtime_unavailable_display() {
    use vernal_context::ApplicationBuildError;
    // Get a TryCurrentError by calling try_current outside a runtime
    let source = tokio::runtime::Handle::try_current().unwrap_err();
    let err = ApplicationBuildError::TokioRuntimeUnavailable { source };
    let display = format!("{err}");
    assert!(display.contains("Tokio runtime is unavailable"));
}

#[test]
fn build_error_tokio_runtime_unavailable_source() {
    use vernal_context::ApplicationBuildError;
    let source = tokio::runtime::Handle::try_current().unwrap_err();
    let err = ApplicationBuildError::TokioRuntimeUnavailable { source };
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn build_error_definition_display() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::DefinitionError;
    let err = ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier { value: "bad".into() },
    };
    let display = format!("{err}");
    assert!(display.contains("definition is invalid"));
}

#[test]
fn build_error_definition_source() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::DefinitionError;
    let err = ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier { value: "bad".into() },
    };
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn build_error_graph_display() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::GraphError;
    let err = ApplicationBuildError::Graph {
        source: GraphError::MissingDependency { path: vec!["a".into(), "b".into()] },
    };
    let display = format!("{err}");
    assert!(display.contains("dependency graph is invalid"));
}

#[test]
fn build_error_graph_source() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::GraphError;
    let err = ApplicationBuildError::Graph {
        source: GraphError::MissingDependency { path: vec!["a".into()] },
    };
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn build_error_context_display() {
    use vernal_context::ApplicationBuildError;
    let err = ApplicationBuildError::Context {
        source: ContextError::InvalidState {
            operation: "test",
            state: ContextState::Created,
        },
    };
    let display = format!("{err}");
    assert!(display.contains("context cannot be built"));
}

#[test]
fn build_error_context_source() {
    use vernal_context::ApplicationBuildError;
    let err = ApplicationBuildError::Context {
        source: ContextError::InvalidState {
            operation: "test",
            state: ContextState::Created,
        },
    };
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn build_error_condition_display() {
    use vernal_context::ApplicationBuildError;
    use vernal_context::ConditionError;
    let err = ApplicationBuildError::Condition {
        source: ConditionError::EmptyProfileSet,
    };
    let display = format!("{err}");
    assert!(display.contains("conditional component assembly failed"));
}

#[test]
fn build_error_condition_source() {
    use vernal_context::ApplicationBuildError;
    use vernal_context::ConditionError;
    let err = ApplicationBuildError::Condition {
        source: ConditionError::EmptyProfileSet,
    };
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn build_error_advisor_resolution_display() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::{ComponentKey, ResolveError};
    let err = ApplicationBuildError::AdvisorResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(ResolveError::NotFound {
            component: "adv".into(),
            path: vec![],
        }),
    };
    let display = format!("{err}");
    assert!(display.contains("advisor"));
    assert!(display.contains("cannot be resolved"));
}

#[test]
fn build_error_advisor_resolution_source() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::{ComponentKey, ResolveError};
    let err = ApplicationBuildError::AdvisorResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(ResolveError::NotFound {
            component: "adv".into(),
            path: vec![],
        }),
    };
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn build_error_local_advisor_resolution_display() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::{ComponentKey, ResolveError};
    let err = ApplicationBuildError::LocalAdvisorResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(ResolveError::NotFound {
            component: "ladv".into(),
            path: vec![],
        }),
    };
    let display = format!("{err}");
    assert!(display.contains("local advisor"));
}

#[test]
fn build_error_local_advisor_resolution_source() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::{ComponentKey, ResolveError};
    let err = ApplicationBuildError::LocalAdvisorResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(ResolveError::NotFound {
            component: "ladv".into(),
            path: vec![],
        }),
    };
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn build_error_advisor_scope_display() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::ComponentKey;
    let err = ApplicationBuildError::AdvisorScope {
        component: ComponentKey::of::<String>(),
        scope: "request",
    };
    let display = format!("{err}");
    assert!(display.contains("must be singleton"));
    assert!(display.contains("request"));
}

#[test]
fn build_error_advisor_scope_source_none() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::ComponentKey;
    let err = ApplicationBuildError::AdvisorScope {
        component: ComponentKey::of::<String>(),
        scope: "request",
    };
    assert!(std::error::Error::source(&err).is_none());
}

#[test]
fn build_error_operation_metadata_display() {
    use vernal_context::ApplicationBuildError;
    use vernal_aop::{Operation, OperationMetadata, OperationMetadataConflictError};
    let op = Operation::new("comp", "method");
    let existing = OperationMetadata::empty();
    let duplicate = OperationMetadata::empty();
    let err = ApplicationBuildError::OperationMetadata {
        source: OperationMetadataConflictError::new(op, existing, duplicate),
    };
    let display = format!("{err}");
    assert!(display.contains("metadata is inconsistent"));
}

#[test]
fn build_error_operation_metadata_source() {
    use vernal_context::ApplicationBuildError;
    use vernal_aop::{Operation, OperationMetadata, OperationMetadataConflictError};
    let op = Operation::new("comp", "method");
    let existing = OperationMetadata::empty();
    let duplicate = OperationMetadata::empty();
    let err = ApplicationBuildError::OperationMetadata {
        source: OperationMetadataConflictError::new(op, existing, duplicate),
    };
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn build_error_aop_catalog_display() {
    use vernal_context::ApplicationBuildError;
    use vernal_aop::InvocationPlanCatalogInitializationError;
    let err = ApplicationBuildError::AopCatalogInitialization {
        source: InvocationPlanCatalogInitializationError,
    };
    let display = format!("{err}");
    assert!(display.contains("AOP catalog cannot be initialized"));
}

#[test]
fn build_error_aop_catalog_source() {
    use vernal_context::ApplicationBuildError;
    use vernal_aop::InvocationPlanCatalogInitializationError;
    let err = ApplicationBuildError::AopCatalogInitialization {
        source: InvocationPlanCatalogInitializationError,
    };
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn build_error_from_definition_error() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::DefinitionError;
    let err: ApplicationBuildError = DefinitionError::InvalidQualifier { value: "v".into() }.into();
    let display = format!("{err}");
    assert!(display.contains("definition is invalid"));
}

#[test]
fn build_error_from_graph_error() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::GraphError;
    let err: ApplicationBuildError = GraphError::MissingDependency { path: vec![] }.into();
    let display = format!("{err}");
    assert!(display.contains("dependency graph is invalid"));
}

#[test]
fn build_error_from_context_error() {
    use vernal_context::ApplicationBuildError;
    let err: ApplicationBuildError = ContextError::LifecycleCancelled { operation: "r" }.into();
    let display = format!("{err}");
    assert!(display.contains("context cannot be built"));
}

#[test]
fn build_error_from_condition_error() {
    use vernal_context::ApplicationBuildError;
    use vernal_context::ConditionError;
    let err: ApplicationBuildError = ConditionError::EmptyProfileSet.into();
    let display = format!("{err}");
    assert!(display.contains("conditional component assembly failed"));
}

#[test]
fn build_error_from_operation_metadata_conflict() {
    use vernal_context::ApplicationBuildError;
    use vernal_aop::{Operation, OperationMetadata, OperationMetadataConflictError};
    let op = Operation::new("c", "m");
    let err: ApplicationBuildError = OperationMetadataConflictError::new(
        op, OperationMetadata::empty(), OperationMetadata::empty(),
    ).into();
    let display = format!("{err}");
    assert!(display.contains("metadata is inconsistent"));
}

#[test]
fn build_error_from_invocation_plan_catalog() {
    use vernal_context::ApplicationBuildError;
    use vernal_aop::InvocationPlanCatalogInitializationError;
    let err: ApplicationBuildError = InvocationPlanCatalogInitializationError.into();
    let display = format!("{err}");
    assert!(display.contains("AOP catalog cannot be initialized"));
}

#[test]
fn build_error_debug_all_variants() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::{DefinitionError, GraphError, ComponentKey, ResolveError};
    use vernal_aop::{Operation, OperationMetadata, OperationMetadataConflictError, InvocationPlanCatalogInitializationError};

    let errs = vec![
        format!("{:?}", ApplicationBuildError::TokioRuntimeUnavailable { source: tokio::runtime::Handle::try_current().unwrap_err() }),
        format!("{:?}", ApplicationBuildError::Definition { source: DefinitionError::InvalidQualifier { value: "v".into() } }),
        format!("{:?}", ApplicationBuildError::Graph { source: GraphError::MissingDependency { path: vec![] } }),
        format!("{:?}", ApplicationBuildError::Context { source: ContextError::InvalidState { operation: "o", state: ContextState::Created } }),
        format!("{:?}", ApplicationBuildError::Condition { source: vernal_context::ConditionError::EmptyProfileSet }),
        format!("{:?}", ApplicationBuildError::AdvisorResolution { component: ComponentKey::of::<String>(), source: Box::new(ResolveError::NotFound { component: "c".into(), path: vec![] }) }),
        format!("{:?}", ApplicationBuildError::LocalAdvisorResolution { component: ComponentKey::of::<String>(), source: Box::new(ResolveError::NotFound { component: "c".into(), path: vec![] }) }),
        format!("{:?}", ApplicationBuildError::AdvisorScope { component: ComponentKey::of::<String>(), scope: "s" }),
        format!("{:?}", ApplicationBuildError::OperationMetadata { source: OperationMetadataConflictError::new(Operation::new("c", "m"), OperationMetadata::empty(), OperationMetadata::empty()) }),
        format!("{:?}", ApplicationBuildError::AopCatalogInitialization { source: InvocationPlanCatalogInitializationError }),
    ];
    for d in errs {
        assert!(!d.is_empty());
    }
}

// ════════════════════════════════════════════════════════════════════
// ApplicationModuleRegistrar - 更多路径覆盖
// ════════════════════════════════════════════════════════════════════

struct TrueCond;
impl ComponentCondition for TrueCond {
    fn name(&self) -> &'static str { "true" }
    fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> { Ok(true) }
}

#[test]
fn registrar_populate_all_fields() {
    let mut reg = ApplicationModuleRegistrar::new();
    reg.register(vernal_beans::ComponentDefinition::shared_value::<i32>(1));
    reg.register_all(vec![vernal_beans::ComponentDefinition::shared_value::<u32>(2)]);

    let binding = vernal_beans::TraitBinding::new::<dyn std::fmt::Display + Send + Sync, String, _>(
        |arc_string: Arc<String>| -> Arc<dyn std::fmt::Display + Send + Sync> { arc_string },
    );
    reg.bind(binding);
    reg.bind_all(vec![]);

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

    let source = vernal_context::MapPropertySource::new("app", [("k", "v")]).unwrap();
    reg.property_source_first(Arc::new(source));
    let source2 = vernal_context::MapPropertySource::new("app2", [("k2", "v2")]).unwrap();
    reg.property_source_last(Arc::new(source2));
    reg.active_profile("prod");
    reg.default_profile("default");

    use vernal_aop::Operation;
    reg.operation(Operation::new("comp", "m1"));
    reg.operations(vec![Operation::new("comp", "m2"), Operation::new("comp", "m3")]);

    let module = ConditionalComponentModule::new("test-mod", TrueCond);
    reg.conditional(module);
    let module2 = ConditionalComponentModule::new("test-mod2", TrueCond);
    reg.conditionals(vec![module2]);
}

// ════════════════════════════════════════════════════════════════════
// ApplicationContext - metadata getters/setters
// ════════════════════════════════════════════════════════════════════

#[test]
fn context_state_as_str() {
    assert_eq!(ContextState::Created.as_str(), "created");
    assert_eq!(ContextState::Refreshing.as_str(), "refreshing");
    assert_eq!(ContextState::Refreshed.as_str(), "refreshed");
    assert_eq!(ContextState::Starting.as_str(), "starting");
    assert_eq!(ContextState::Ready.as_str(), "ready");
    assert_eq!(ContextState::RollingBack.as_str(), "rolling_back");
    assert_eq!(ContextState::Draining.as_str(), "draining");
    assert_eq!(ContextState::Failed.as_str(), "failed");
    assert_eq!(ContextState::Pausing.as_str(), "pausing");
    assert_eq!(ContextState::Paused.as_str(), "paused");
    assert_eq!(ContextState::Closed.as_str(), "closed");
}

#[test]
fn context_state_debug() {
    assert!(!format!("{:?}", ContextState::Created).is_empty());
    assert!(!format!("{:?}", ContextState::Ready).is_empty());
}

#[test]
fn context_state_clone() {
    let state = ContextState::Ready;
    let cloned = state.clone();
    assert_eq!(state, cloned);
}

#[test]
fn context_state_default() {
    let state = ContextState::default();
    assert_eq!(state, ContextState::Created);
}

#[test]
fn context_state_eq() {
    assert_eq!(ContextState::Created, ContextState::Created);
    assert_ne!(ContextState::Created, ContextState::Ready);
}

// ════════════════════════════════════════════════════════════════════
// ManagedTaskError - 更多 Display 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn managed_task_error_all_display() {
    use vernal_context::ManagedTaskError;
    use vernal_core::SharedError;

    let errors = vec![
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
    for d in errors {
        assert!(!d.is_empty());
    }
}

#[test]
fn managed_task_error_debug_all() {
    use vernal_context::ManagedTaskError;
    use vernal_core::SharedError;

    let errors = vec![
        format!("{:?}", ManagedTaskError::InvalidName),
        format!("{:?}", ManagedTaskError::IdentifierExhausted { task: "t" }),
        format!("{:?}", ManagedTaskError::SpawnRejected { task: "t" }),
        format!("{:?}", ManagedTaskError::TaskFailed { task: "t", source: Arc::new(std::io::Error::other("e")) as SharedError }),
        format!("{:?}", ManagedTaskError::TaskPanicked { task: "t", source: Arc::new(std::io::Error::other("e")) as SharedError }),
        format!("{:?}", ManagedTaskError::TaskCancelled { task: "t", source: Arc::new(std::io::Error::other("e")) as SharedError }),
        format!("{:?}", ManagedTaskError::ShutdownTimeout { timeout: Duration::from_secs(5), remaining: 3 }),
        format!("{:?}", ManagedTaskError::AbortTimeout { timeout: Duration::from_secs(5), remaining: 2 }),
        format!("{:?}", ManagedTaskError::CoordinatorUnavailable { remaining: 1 }),
    ];
    for d in errors {
        assert!(!d.is_empty());
    }
}

#[test]
fn managed_task_error_source_chain() {
    use vernal_context::ManagedTaskError;
    use vernal_core::SharedError;

    // Variants with source
    let e1 = ManagedTaskError::TaskFailed { task: "t", source: Arc::new(std::io::Error::other("e")) as SharedError };
    assert!(std::error::Error::source(&e1).is_some());

    let e2 = ManagedTaskError::TaskPanicked { task: "t", source: Arc::new(std::io::Error::other("e")) as SharedError };
    assert!(std::error::Error::source(&e2).is_some());

    let e3 = ManagedTaskError::TaskCancelled { task: "t", source: Arc::new(std::io::Error::other("e")) as SharedError };
    assert!(std::error::Error::source(&e3).is_some());

    // Variants without source
    assert!(std::error::Error::source(&ManagedTaskError::InvalidName).is_none());
    assert!(std::error::Error::source(&ManagedTaskError::IdentifierExhausted { task: "t" }).is_none());
    assert!(std::error::Error::source(&ManagedTaskError::SpawnRejected { task: "t" }).is_none());
    assert!(std::error::Error::source(&ManagedTaskError::ShutdownTimeout { timeout: Duration::from_secs(5), remaining: 3 }).is_none());
    assert!(std::error::Error::source(&ManagedTaskError::AbortTimeout { timeout: Duration::from_secs(5), remaining: 2 }).is_none());
    assert!(std::error::Error::source(&ManagedTaskError::CoordinatorUnavailable { remaining: 1 }).is_none());
}

// ════════════════════════════════════════════════════════════════════
// ApplicationRunnerFailure
// ════════════════════════════════════════════════════════════════════

#[test]
fn application_runner_failure_display() {
    use vernal_context::ApplicationRunnerFailure;
    let err = ApplicationRunnerFailure::new("my-runner", Arc::new(std::io::Error::other("fail")) as vernal_core::SharedError);
    let display = format!("{err}");
    assert!(display.contains("my-runner"));
}

#[test]
fn application_runner_failure_debug() {
    use vernal_context::ApplicationRunnerFailure;
    let err = ApplicationRunnerFailure::new("my-runner", Arc::new(std::io::Error::other("fail")) as vernal_core::SharedError);
    let debug = format!("{err:?}");
    assert!(debug.contains("redacted"));
}

#[test]
fn application_runner_failure_source() {
    use vernal_context::ApplicationRunnerFailure;
    let err = ApplicationRunnerFailure::new("my-runner", Arc::new(std::io::Error::other("fail")) as vernal_core::SharedError);
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn application_runner_failure_runner_accessor() {
    use vernal_context::ApplicationRunnerFailure;
    let err = ApplicationRunnerFailure::new("my-runner", Arc::new(std::io::Error::other("fail")) as vernal_core::SharedError);
    assert_eq!(err.runner(), "my-runner");
}

#[test]
fn application_runner_failure_clone() {
    use vernal_context::ApplicationRunnerFailure;
    let err = ApplicationRunnerFailure::new("my-runner", Arc::new(std::io::Error::other("fail")) as vernal_core::SharedError);
    let cloned = err.clone();
    assert_eq!(cloned.runner(), "my-runner");
}

// ════════════════════════════════════════════════════════════════════
// ApplicationRunnerFailure in ContextError
// ════════════════════════════════════════════════════════════════════

#[test]
fn context_error_application_runner_failed_display() {
    use vernal_context::ApplicationRunnerFailure;
    let err = ContextError::ApplicationRunnerFailed {
        source: ApplicationRunnerFailure::new("r", Arc::new(std::io::Error::other("f")) as vernal_core::SharedError),
    };
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn context_error_application_runner_failed_debug() {
    use vernal_context::ApplicationRunnerFailure;
    let err = ContextError::ApplicationRunnerFailed {
        source: ApplicationRunnerFailure::new("r", Arc::new(std::io::Error::other("f")) as vernal_core::SharedError),
    };
    let debug = format!("{err:?}");
    assert!(debug.contains("ApplicationRunnerFailed"));
}

// ════════════════════════════════════════════════════════════════════
// ContextError source() 方法
// ════════════════════════════════════════════════════════════════════

#[test]
fn context_error_source_chain() {
    use vernal_core::SharedError;
    use vernal_context::ApplicationRunnerFailure;

    // Variants with source
    let e1 = ContextError::Lifecycle {
        component: "c",
        phase: LifecyclePhase::Start,
        source: Arc::new(std::io::Error::other("e")) as SharedError,
    };
    assert!(std::error::Error::source(&e1).is_some());

    let e2 = ContextError::LifecycleCoordinator {
        operation: "o",
        source: Arc::new(std::io::Error::other("e")) as SharedError,
    };
    assert!(std::error::Error::source(&e2).is_some());

    let e3 = ContextError::ShutdownSignal {
        source: Arc::new(std::io::Error::other("e")) as SharedError,
    };
    assert!(std::error::Error::source(&e3).is_some());

    // PauseRestart source returns None (per Error impl: _ => None branch)
    let e4 = ContextError::PauseRestart {
        operation: "o",
        component: "c",
        phase: LifecyclePhase::Pause,
        source: Arc::new(std::io::Error::other("e")) as SharedError,
    };
    assert!(std::error::Error::source(&e4).is_none());

    let e5 = ContextError::ApplicationRunnerFailed {
        source: ApplicationRunnerFailure::new("r", Arc::new(std::io::Error::other("f")) as SharedError),
    };
    assert!(std::error::Error::source(&e5).is_some());

    // Variants without source
    assert!(std::error::Error::source(&ContextError::InvalidState { operation: "o", state: ContextState::Created }).is_none());
    assert!(std::error::Error::source(&ContextError::LifecycleCancelled { operation: "o" }).is_none());
    assert!(std::error::Error::source(&ContextError::LifecycleTimeout {
        component: "c", phase: LifecyclePhase::Start, timeout: Duration::from_secs(5), abort_settled: false,
    }).is_none());
    assert!(std::error::Error::source(&ContextError::ApplicationRunnerTimeout {
        runner: "r", timeout: Duration::from_secs(5), abort_settled: false,
    }).is_none());

    // ManagedTask has source
    let e6 = ContextError::ManagedTask { source: ManagedTaskError::InvalidName };
    assert!(std::error::Error::source(&e6).is_some());

    // Resolution errors have source
    let e7 = ContextError::ContainerWarmUp {
        source: Box::new(vernal_beans::ResolveError::NotFound { component: "c".into(), path: vec![] }),
    };
    assert!(std::error::Error::source(&e7).is_some());
}

// ════════════════════════════════════════════════════════════════════
// ContextError - 更多 Display 路径 (resolution errors)
// ════════════════════════════════════════════════════════════════════

#[test]
fn context_error_container_warm_up_display() {
    use vernal_beans::ResolveError;
    let err = ContextError::ContainerWarmUp {
        source: Box::new(ResolveError::NotFound { component: "c".into(), path: vec![] }),
    };
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn context_error_component_resolution_display() {
    use vernal_beans::{ComponentKey, ResolveError};
    let err = ContextError::ComponentResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(ResolveError::NotFound { component: "c".into(), path: vec![] }),
    };
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn context_error_event_listener_resolution_display() {
    use vernal_beans::{ComponentKey, ResolveError};
    let err = ContextError::EventListenerResolution {
        component: ComponentKey::of::<String>(),
        event: "OrderCreated",
        source: Box::new(ResolveError::NotFound { component: "c".into(), path: vec![] }),
    };
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn context_error_application_runner_resolution_display() {
    use vernal_beans::{ComponentKey, ResolveError};
    let err = ContextError::ApplicationRunnerResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(ResolveError::NotFound { component: "c".into(), path: vec![] }),
    };
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn context_error_scheduled_task_resolution_display() {
    use vernal_beans::{ComponentKey, ResolveError};
    let err = ContextError::ScheduledTaskResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(ResolveError::NotFound { component: "c".into(), path: vec![] }),
    };
    let display = format!("{err}");
    assert!(!display.is_empty());
}

// ════════════════════════════════════════════════════════════════════
// ContextError clone
// ════════════════════════════════════════════════════════════════════

#[test]
fn context_error_clone() {
    use vernal_core::SharedError;
    let err = ContextError::LifecycleCoordinator {
        operation: "o",
        source: Arc::new(std::io::Error::other("e")) as SharedError,
    };
    let cloned = err.clone();
    let display = format!("{cloned}");
    assert!(!display.is_empty());
}

// ════════════════════════════════════════════════════════════════════
// ResolveError Display 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn resolve_error_all_display() {
    use vernal_beans::{ComponentKey, TraitKey, ScopeKey, ScopeState, ResolveError};
    use vernal_core::SharedError;

    let errors = vec![
        format!("{}", ResolveError::NotFound { component: "c".into(), path: vec![] }),
        format!("{}", ResolveError::Ambiguous { component: "c".into(), candidates: vec!["a".into()], path: vec![] }),
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
    for d in errors {
        assert!(!d.is_empty());
    }
}

#[test]
fn resolve_error_debug_all() {
    use vernal_beans::{ComponentKey, TraitKey, ScopeKey, ScopeState, ResolveError};
    use vernal_core::SharedError;

    let errors = vec![
        format!("{:?}", ResolveError::NotFound { component: "c".into(), path: vec![] }),
        format!("{:?}", ResolveError::Ambiguous { component: "c".into(), candidates: vec![], path: vec![] }),
        format!("{:?}", ResolveError::UndeclaredDependency { component: ComponentKey::of::<String>(), dependency: "d".into() }),
        format!("{:?}", ResolveError::TypeMismatch { component: ComponentKey::of::<String>() }),
        format!("{:?}", ResolveError::TraitBindingTypeMismatch { binding: TraitKey::of::<dyn std::fmt::Display>(), target: ComponentKey::of::<String>() }),
        format!("{:?}", ResolveError::Construction { component: ComponentKey::of::<String>(), source: Arc::new(std::io::Error::other("e")) as SharedError }),
        format!("{:?}", ResolveError::CircularRuntime { path: vec![] }),
        format!("{:?}", ResolveError::ProviderUsedDuringConstruction { component: ComponentKey::of::<String>(), dependency: "d".into() }),
        format!("{:?}", ResolveError::ScopeNotActive { component: ComponentKey::of::<String>(), scope: ScopeKey::of::<String>() }),
        format!("{:?}", ResolveError::ScopeOwnerMismatch { scope: ScopeKey::of::<String>() }),
        format!("{:?}", ResolveError::ScopeUnavailable { component: ComponentKey::of::<String>(), scope: ScopeKey::of::<String>(), state: ScopeState::Closed, cancelled: true }),
    ];
    for d in errors {
        assert!(!d.is_empty());
    }
}

#[test]
fn resolve_error_source_chain() {
    use vernal_beans::{ComponentKey, ResolveError};
    use vernal_core::SharedError;

    // Construction has source
    let e1 = ResolveError::Construction {
        component: ComponentKey::of::<String>(),
        source: Arc::new(std::io::Error::other("e")) as SharedError,
    };
    assert!(std::error::Error::source(&e1).is_some());

    // Others don't
    assert!(std::error::Error::source(&ResolveError::NotFound { component: "c".into(), path: vec![] }).is_none());
    assert!(std::error::Error::source(&ResolveError::TypeMismatch { component: ComponentKey::of::<String>() }).is_none());
}

// ════════════════════════════════════════════════════════════════════
// DefinitionError Display
// ════════════════════════════════════════════════════════════════════

#[test]
fn definition_error_all_display() {
    use vernal_beans::{DefinitionError, ComponentKey, TraitKey};

    let errors = vec![
        format!("{}", DefinitionError::InvalidQualifier { value: "bad".into() }),
        format!("{}", DefinitionError::DuplicateDefinition { key: ComponentKey::of::<String>() }),
        format!("{}", DefinitionError::DuplicateTraitBinding { key: TraitKey::of::<dyn std::fmt::Display>(), target: ComponentKey::of::<String>() }),
        format!("{}", DefinitionError::DuplicateQualifiedTraitBinding { key: TraitKey::of::<dyn std::fmt::Display>() }),
        format!("{}", DefinitionError::MultiplePrimaryTraitBindings { trait_name: "Display" }),
    ];
    for d in errors {
        assert!(!d.is_empty());
    }
}

// ════════════════════════════════════════════════════════════════════
// GraphError Display
// ════════════════════════════════════════════════════════════════════

#[test]
fn graph_error_all_display() {
    use vernal_beans::GraphError;

    let errors = vec![
        format!("{}", GraphError::MissingDependency { path: vec!["a".into(), "b".into()] }),
        format!("{}", GraphError::AmbiguousDependency { path: vec!["a".into()], candidates: vec!["x".into(), "y".into()] }),
        format!("{}", GraphError::MissingTraitBindingTarget { binding: "b".into() }),
        format!("{}", GraphError::Cycle { path: vec!["a".into(), "b".into(), "a".into()] }),
    ];
    for d in errors {
        assert!(!d.is_empty());
    }
}

// ════════════════════════════════════════════════════════════════════
// ApplicationStartupCoordinator - upgrade_pause_error
// ════════════════════════════════════════════════════════════════════

// upgrade_pause_error is pub(crate) so we test through ContextError variants
// that exercise the same display paths

#[test]
fn context_error_pause_restart_with_lifecycle_source() {
    use vernal_core::SharedError;
    let err = ContextError::PauseRestart {
        operation: "pause",
        component: "db",
        phase: LifecyclePhase::Pause,
        source: Arc::new(ContextError::Lifecycle {
            component: "db",
            phase: LifecyclePhase::Pause,
            source: Arc::new(std::io::Error::other("inner")) as SharedError,
        }) as SharedError,
    };
    let display = format!("{err}");
    assert!(display.contains("pause"));
}

// ════════════════════════════════════════════════════════════════════
// ScopeKey, ScopeState, TraitKey
// ════════════════════════════════════════════════════════════════════

#[test]
fn scope_key_of() {
    use vernal_beans::ScopeKey;
    let key = ScopeKey::of::<String>();
    let debug = format!("{:?}", key);
    assert!(!debug.is_empty());
}

#[test]
fn scope_state_debug() {
    use vernal_beans::ScopeState;
    assert!(!format!("{:?}", ScopeState::Open).is_empty());
    assert!(!format!("{:?}", ScopeState::Closing).is_empty());
    assert!(!format!("{:?}", ScopeState::Closed).is_empty());
}

#[test]
fn scope_state_default() {
    use vernal_beans::ScopeState;
    assert_eq!(ScopeState::default(), ScopeState::Open);
}

#[test]
fn trait_key_of() {
    use vernal_beans::TraitKey;
    let key = TraitKey::of::<dyn std::fmt::Display>();
    let debug = format!("{:?}", key);
    assert!(!debug.is_empty());
}

// ════════════════════════════════════════════════════════════════════
// OperationMetadata
// ════════════════════════════════════════════════════════════════════

#[test]
fn operation_metadata_empty() {
    use vernal_aop::OperationMetadata;
    let meta = OperationMetadata::empty();
    let debug = format!("{:?}", meta);
    assert!(!debug.is_empty());
}

#[test]
fn operation_metadata_with_tag() {
    use vernal_aop::OperationMetadata;
    let meta = OperationMetadata::empty().with_tag("tag").unwrap();
    let debug = format!("{:?}", meta);
    assert!(!debug.is_empty());
}

#[test]
fn operation_metadata_with_qualifier() {
    use vernal_aop::OperationMetadata;
    let meta = OperationMetadata::empty().with_qualifier("qual").unwrap();
    let debug = format!("{:?}", meta);
    assert!(!debug.is_empty());
}

// ════════════════════════════════════════════════════════════════════
// ApplicationEnvironment - 更多路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn environment_get_typed_missing() {
    let env = ApplicationEnvironment::builder().build();
    let result: Result<Option<i32>, _> = env.get("missing");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn environment_get_typed_present() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new("app", [("port", "8080")]).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let result: Result<Option<u16>, _> = env.get("port");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(8080));
}

#[test]
fn environment_get_typed_invalid() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new("app", [("port", "not-a-number")]).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let result: Result<Option<u16>, _> = env.get("port");
    assert!(result.is_err());
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
fn environment_require_typed_missing() {
    let env = ApplicationEnvironment::builder().build();
    let result: Result<u16, _> = env.require("missing");
    assert!(result.is_err());
}

#[test]
fn environment_property_source_priority() {
    let mut builder = ApplicationEnvironment::builder();
    // Add source with add_first (high priority)
    let high = vernal_context::MapPropertySource::new("high", [("k", "high_val")]).unwrap();
    builder.add_first(Arc::new(high)).unwrap();
    let low = vernal_context::MapPropertySource::new("low", [("k", "low_val")]).unwrap();
    builder.add_last(Arc::new(low)).unwrap();
    let env = builder.build();

    let val = env.property("k").unwrap().unwrap();
    assert_eq!(val, "high_val");
}
