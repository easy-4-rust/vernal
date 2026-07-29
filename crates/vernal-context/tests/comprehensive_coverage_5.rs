//! 全面覆盖测试 - Part 5: 针对 context_error fmt_application_runner/fmt_scheduled_task/fmt_event_listener
//! 以及 application_context, application_module_registrar 等文件的剩余未覆盖路径。

use std::sync::Arc;
use std::time::Duration;

use vernal_context::{
    ApplicationEnvironment, ApplicationModuleRegistrar,
    ConditionalComponentModule, ComponentCondition,
    ContextError, ContextState, LifecyclePhase,
    ConfigurationPhase, ManagedTaskError,
    ApplicationRunnerFailure,
};
use vernal_core::SharedError;
use vernal_beans::ComponentKey;

// ════════════════════════════════════════════════════════════════════
// ContextError - fmt_application_runner 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn context_error_runner_definition_not_found_display() {
    let err = ContextError::ApplicationRunnerDefinitionNotFound {
        component: ComponentKey::of::<String>(),
    };
    let display = format!("{err}");
    assert!(display.contains("runner"));
    assert!(display.contains("not registered"));
}

#[test]
fn context_error_runner_scope_display() {
    let err = ContextError::ApplicationRunnerScope {
        component: ComponentKey::of::<String>(),
        scope: "request",
    };
    let display = format!("{err}");
    assert!(display.contains("runner"));
    assert!(display.contains("request"));
}

#[test]
fn context_error_duplicate_runner_display() {
    let err = ContextError::DuplicateApplicationRunner {
        component: ComponentKey::of::<String>(),
    };
    let display = format!("{err}");
    assert!(display.contains("runner"));
    assert!(display.contains("more than once"));
}

#[test]
fn context_error_runner_resolution_display() {
    let err = ContextError::ApplicationRunnerResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(vernal_beans::ResolveError::NotFound {
            component: "r".into(),
            path: vec![],
        }),
    };
    let display = format!("{err}");
    assert!(display.contains("runner"));
    assert!(display.contains("resolve"));
}

#[test]
fn context_error_runner_failed_display() {
    let err = ContextError::ApplicationRunnerFailed {
        source: ApplicationRunnerFailure::new(
            "my-runner",
            Arc::new(std::io::Error::other("fail")) as SharedError,
        ),
    };
    let display = format!("{err}");
    assert!(display.contains("my-runner"));
}

#[test]
fn context_error_runner_timeout_display() {
    let err = ContextError::ApplicationRunnerTimeout {
        runner: "my-runner",
        timeout: Duration::from_secs(30),
        abort_settled: true,
    };
    let display = format!("{err}");
    assert!(display.contains("my-runner"));
    assert!(display.contains("timeout"));
}

// ════════════════════════════════════════════════════════════════════
// ContextError - fmt_scheduled_task 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn context_error_task_definition_not_found_display() {
    let err = ContextError::ScheduledTaskDefinitionNotFound {
        component: ComponentKey::of::<String>(),
    };
    let display = format!("{err}");
    assert!(display.contains("task"));
    assert!(display.contains("not registered"));
}

#[test]
fn context_error_task_scope_display() {
    let err = ContextError::ScheduledTaskScope {
        component: ComponentKey::of::<String>(),
        scope: "request",
    };
    let display = format!("{err}");
    assert!(display.contains("task"));
    assert!(display.contains("request"));
}

#[test]
fn context_error_duplicate_task_display() {
    let err = ContextError::DuplicateScheduledTask {
        component: ComponentKey::of::<String>(),
    };
    let display = format!("{err}");
    assert!(display.contains("task"));
    assert!(display.contains("more than once"));
}

#[test]
fn context_error_task_resolution_display() {
    let err = ContextError::ScheduledTaskResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(vernal_beans::ResolveError::NotFound {
            component: "t".into(),
            path: vec![],
        }),
    };
    let display = format!("{err}");
    assert!(display.contains("task"));
    assert!(display.contains("resolve"));
}

// ════════════════════════════════════════════════════════════════════
// ContextError - fmt_event_listener 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn context_error_listener_definition_not_found_display() {
    let err = ContextError::EventListenerDefinitionNotFound {
        component: ComponentKey::of::<String>(),
        event: "OrderCreated",
    };
    let display = format!("{err}");
    assert!(display.contains("listener"));
    assert!(display.contains("not registered"));
    assert!(display.contains("OrderCreated"));
}

#[test]
fn context_error_listener_scope_display() {
    let err = ContextError::EventListenerScope {
        component: ComponentKey::of::<String>(),
        event: "OrderCreated",
        scope: "request",
    };
    let display = format!("{err}");
    assert!(display.contains("listener"));
    assert!(display.contains("request"));
}

#[test]
fn context_error_duplicate_listener_display() {
    let err = ContextError::DuplicateEventListener {
        component: ComponentKey::of::<String>(),
        event: "OrderCreated",
    };
    let display = format!("{err}");
    assert!(display.contains("listener"));
    assert!(display.contains("more than once"));
}

// ════════════════════════════════════════════════════════════════════
// ContextError - 主 match 分支
// ════════════════════════════════════════════════════════════════════

#[test]
fn context_error_lifecycle_definition_not_found_display() {
    let err = ContextError::LifecycleDefinitionNotFound {
        component: ComponentKey::of::<String>(),
    };
    let display = format!("{err}");
    assert!(display.contains("lifecycle"));
    assert!(display.contains("not registered"));
}

#[test]
fn context_error_container_warm_up_display() {
    let err = ContextError::ContainerWarmUp {
        source: Box::new(vernal_beans::ResolveError::NotFound {
            component: "c".into(),
            path: vec![],
        }),
    };
    let display = format!("{err}");
    assert!(display.contains("warm up"));
}

#[test]
fn context_error_component_resolution_display() {
    let err = ContextError::ComponentResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(vernal_beans::ResolveError::NotFound {
            component: "c".into(),
            path: vec![],
        }),
    };
    let display = format!("{err}");
    assert!(display.contains("resolve"));
}

#[test]
fn context_error_event_listener_resolution_display() {
    let err = ContextError::EventListenerResolution {
        component: ComponentKey::of::<String>(),
        event: "OrderCreated",
        source: Box::new(vernal_beans::ResolveError::NotFound {
            component: "l".into(),
            path: vec![],
        }),
    };
    let display = format!("{err}");
    assert!(display.contains("listener"));
    assert!(display.contains("OrderCreated"));
}

#[test]
fn context_error_application_runner_resolution_display() {
    let err = ContextError::ApplicationRunnerResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(vernal_beans::ResolveError::NotFound {
            component: "r".into(),
            path: vec![],
        }),
    };
    let display = format!("{err}");
    assert!(display.contains("runner"));
}

#[test]
fn context_error_scheduled_task_resolution_display() {
    let err = ContextError::ScheduledTaskResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(vernal_beans::ResolveError::NotFound {
            component: "t".into(),
            path: vec![],
        }),
    };
    let display = format!("{err}");
    assert!(display.contains("task"));
}

// ════════════════════════════════════════════════════════════════════
// ContextError Debug 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn context_error_debug_runner_variants() {
    let errs = vec![
        format!("{:?}", ContextError::ApplicationRunnerDefinitionNotFound { component: ComponentKey::of::<String>() }),
        format!("{:?}", ContextError::ApplicationRunnerScope { component: ComponentKey::of::<String>(), scope: "s" }),
        format!("{:?}", ContextError::DuplicateApplicationRunner { component: ComponentKey::of::<String>() }),
        format!("{:?}", ContextError::ApplicationRunnerResolution { component: ComponentKey::of::<String>(), source: Box::new(vernal_beans::ResolveError::NotFound { component: "r".into(), path: vec![] }) }),
        format!("{:?}", ContextError::ApplicationRunnerFailed { source: ApplicationRunnerFailure::new("r", Arc::new(std::io::Error::other("f")) as SharedError) }),
        format!("{:?}", ContextError::ApplicationRunnerTimeout { runner: "r", timeout: Duration::from_secs(5), abort_settled: false }),
    ];
    for d in errs {
        assert!(!d.is_empty());
    }
}

#[test]
fn context_error_debug_task_variants() {
    let errs = vec![
        format!("{:?}", ContextError::ScheduledTaskDefinitionNotFound { component: ComponentKey::of::<String>() }),
        format!("{:?}", ContextError::ScheduledTaskScope { component: ComponentKey::of::<String>(), scope: "s" }),
        format!("{:?}", ContextError::DuplicateScheduledTask { component: ComponentKey::of::<String>() }),
        format!("{:?}", ContextError::ScheduledTaskResolution { component: ComponentKey::of::<String>(), source: Box::new(vernal_beans::ResolveError::NotFound { component: "t".into(), path: vec![] }) }),
    ];
    for d in errs {
        assert!(!d.is_empty());
    }
}

#[test]
fn context_error_debug_listener_variants() {
    let errs = vec![
        format!("{:?}", ContextError::EventListenerDefinitionNotFound { component: ComponentKey::of::<String>(), event: "e" }),
        format!("{:?}", ContextError::EventListenerScope { component: ComponentKey::of::<String>(), event: "e", scope: "s" }),
        format!("{:?}", ContextError::DuplicateEventListener { component: ComponentKey::of::<String>(), event: "e" }),
        format!("{:?}", ContextError::EventListenerResolution { component: ComponentKey::of::<String>(), event: "e", source: Box::new(vernal_beans::ResolveError::NotFound { component: "l".into(), path: vec![] }) }),
    ];
    for d in errs {
        assert!(!d.is_empty());
    }
}

#[test]
fn context_error_debug_resolution_variants() {
    let errs = vec![
        format!("{:?}", ContextError::ContainerWarmUp { source: Box::new(vernal_beans::ResolveError::NotFound { component: "c".into(), path: vec![] }) }),
        format!("{:?}", ContextError::ComponentResolution { component: ComponentKey::of::<String>(), source: Box::new(vernal_beans::ResolveError::NotFound { component: "c".into(), path: vec![] }) }),
    ];
    for d in errs {
        assert!(!d.is_empty());
    }
}

// ════════════════════════════════════════════════════════════════════
// ContextError Clone 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn context_error_clone_all_variants() {
    let errs = vec![
        ContextError::InvalidState { operation: "o", state: ContextState::Created },
        ContextError::LifecycleDefinitionNotFound { component: ComponentKey::of::<String>() },
        ContextError::ApplicationRunnerDefinitionNotFound { component: ComponentKey::of::<String>() },
        ContextError::ApplicationRunnerScope { component: ComponentKey::of::<String>(), scope: "s" },
        ContextError::DuplicateApplicationRunner { component: ComponentKey::of::<String>() },
        ContextError::ScheduledTaskDefinitionNotFound { component: ComponentKey::of::<String>() },
        ContextError::ScheduledTaskScope { component: ComponentKey::of::<String>(), scope: "s" },
        ContextError::DuplicateScheduledTask { component: ComponentKey::of::<String>() },
        ContextError::EventListenerDefinitionNotFound { component: ComponentKey::of::<String>(), event: "e" },
        ContextError::EventListenerScope { component: ComponentKey::of::<String>(), event: "e", scope: "s" },
        ContextError::DuplicateEventListener { component: ComponentKey::of::<String>(), event: "e" },
        ContextError::LifecycleCancelled { operation: "o" },
        ContextError::LifecycleTimeout { component: "c", phase: LifecyclePhase::Start, timeout: Duration::from_secs(5), abort_settled: false },
        ContextError::ApplicationRunnerTimeout { runner: "r", timeout: Duration::from_secs(5), abort_settled: false },
    ];
    for err in errs {
        let cloned = err.clone();
        let display = format!("{cloned}");
        assert!(!display.is_empty());
    }
}

// ════════════════════════════════════════════════════════════════════
// ApplicationBuildError - 更多 Display 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn build_error_all_variants_display() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::{DefinitionError, GraphError, ResolveError};
    use vernal_aop::{Operation, OperationMetadata, OperationMetadataConflictError, InvocationPlanCatalogInitializationError};

    let errs = vec![
        format!("{}", ApplicationBuildError::TokioRuntimeUnavailable { source: tokio::runtime::Handle::try_current().unwrap_err() }),
        format!("{}", ApplicationBuildError::Definition { source: DefinitionError::InvalidQualifier { value: "v".into() } }),
        format!("{}", ApplicationBuildError::Graph { source: GraphError::MissingDependency { path: vec![] } }),
        format!("{}", ApplicationBuildError::Context { source: ContextError::InvalidState { operation: "o", state: ContextState::Created } }),
        format!("{}", ApplicationBuildError::Condition { source: vernal_context::ConditionError::EmptyProfileSet }),
        format!("{}", ApplicationBuildError::AdvisorResolution { component: ComponentKey::of::<String>(), source: Box::new(ResolveError::NotFound { component: "c".into(), path: vec![] }) }),
        format!("{}", ApplicationBuildError::LocalAdvisorResolution { component: ComponentKey::of::<String>(), source: Box::new(ResolveError::NotFound { component: "c".into(), path: vec![] }) }),
        format!("{}", ApplicationBuildError::AdvisorScope { component: ComponentKey::of::<String>(), scope: "request" }),
        format!("{}", ApplicationBuildError::OperationMetadata { source: OperationMetadataConflictError::new(Operation::new("c", "m"), OperationMetadata::empty(), OperationMetadata::empty()) }),
        format!("{}", ApplicationBuildError::AopCatalogInitialization { source: InvocationPlanCatalogInitializationError }),
    ];
    for d in errs {
        assert!(!d.is_empty());
    }
}

// ════════════════════════════════════════════════════════════════════
// DefinitionError - 更多 Display 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn definition_error_all_variants_display() {
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

#[test]
fn definition_error_debug_all() {
    use vernal_beans::{DefinitionError, ComponentKey, TraitKey};

    let errs = vec![
        format!("{:?}", DefinitionError::InvalidQualifier { value: "bad".into() }),
        format!("{:?}", DefinitionError::DuplicateDefinition { key: ComponentKey::of::<String>() }),
        format!("{:?}", DefinitionError::DuplicateTraitBinding { key: TraitKey::of::<dyn std::fmt::Display>(), target: ComponentKey::of::<String>() }),
        format!("{:?}", DefinitionError::DuplicateQualifiedTraitBinding { key: TraitKey::of::<dyn std::fmt::Display>() }),
        format!("{:?}", DefinitionError::MultiplePrimaryTraitBindings { trait_name: "Display" }),
    ];
    for d in errs {
        assert!(!d.is_empty());
    }
}

// ════════════════════════════════════════════════════════════════════
// GraphError - 更多 Display 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn graph_error_all_variants_display() {
    use vernal_beans::GraphError;

    let errs = vec![
        format!("{}", GraphError::MissingDependency { path: vec!["a".into(), "b".into()] }),
        format!("{}", GraphError::AmbiguousDependency { path: vec!["a".into()], candidates: vec!["x".into(), "y".into()] }),
        format!("{}", GraphError::MissingTraitBindingTarget { binding: "b".into() }),
        format!("{}", GraphError::Cycle { path: vec!["a".into(), "b".into(), "a".into()] }),
    ];
    for d in errs {
        assert!(!d.is_empty());
    }
}

#[test]
fn graph_error_debug_all() {
    use vernal_beans::GraphError;

    let errs = vec![
        format!("{:?}", GraphError::MissingDependency { path: vec![] }),
        format!("{:?}", GraphError::AmbiguousDependency { path: vec![], candidates: vec![] }),
        format!("{:?}", GraphError::MissingTraitBindingTarget { binding: "b".into() }),
        format!("{:?}", GraphError::Cycle { path: vec![] }),
    ];
    for d in errs {
        assert!(!d.is_empty());
    }
}

// ════════════════════════════════════════════════════════════════════
// ResolveError - 更多路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn resolve_error_all_variants_display() {
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
// ManagedTaskError - 更多路径
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

#[test]
fn managed_task_error_all_debug() {
    use vernal_context::ManagedTaskError;
    use vernal_core::SharedError;

    let errs = vec![
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
    for d in errs {
        assert!(!d.is_empty());
    }
}

// ════════════════════════════════════════════════════════════════════
// ApplicationEnvironment - 更多路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn environment_property_expansion_chain() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new(
        "app",
        [("a", "val_a"), ("b", "${a}_b"), ("c", "${b}_c"), ("d", "${c}_d")]
    ).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let d = env.property("d").unwrap().unwrap();
    assert_eq!(d, "val_a_b_c_d");
}

#[test]
fn environment_property_multiple_in_value() {
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
fn environment_property_default_not_found() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new(
        "app", [("k", "${missing:}")]
    ).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    // Empty default value should be returned
    let val = env.property("k").unwrap().unwrap();
    assert_eq!(val, "");
}

// ════════════════════════════════════════════════════════════════════
// ConditionalComponentModule - 更多路径
// ════════════════════════════════════════════════════════════════════

struct TrueCond;
impl ComponentCondition for TrueCond {
    fn name(&self) -> &'static str { "true" }
    fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> { Ok(true) }
}

struct FalseCond;
impl ComponentCondition for FalseCond {
    fn name(&self) -> &'static str { "false" }
    fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> { Ok(false) }
}

struct FailCond;
impl ComponentCondition for FailCond {
    fn name(&self) -> &'static str { "fail" }
    fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Err(Box::new(std::io::Error::other("condition failed")))
    }
}

#[test]
fn conditional_module_new_with_true_condition() {
    let module = ConditionalComponentModule::new("test", TrueCond);
    assert_eq!(module.name(), "test");
    assert_eq!(module.condition_name(), "true");
}

#[test]
fn conditional_module_new_with_false_condition() {
    let module = ConditionalComponentModule::new("test", FalseCond);
    assert_eq!(module.name(), "test");
    assert_eq!(module.condition_name(), "false");
}

#[test]
fn conditional_module_new_with_fail_condition() {
    let module = ConditionalComponentModule::new("test", FailCond);
    assert_eq!(module.name(), "test");
    assert_eq!(module.condition_name(), "fail");
}

#[test]
fn conditional_module_shared_with_arc() {
    let cond = Arc::new(TrueCond);
    let module = ConditionalComponentModule::shared("test", cond);
    assert_eq!(module.name(), "test");
    assert_eq!(module.condition_name(), "true");
}

#[test]
fn conditional_module_with_phase_register_bean() {
    let module = ConditionalComponentModule::new("test", TrueCond)
        .with_phase(ConfigurationPhase::RegisterBean);
    assert_eq!(module.phase(), ConfigurationPhase::RegisterBean);
}

#[test]
fn conditional_module_debug() {
    let module = ConditionalComponentModule::new("test", TrueCond);
    let debug = format!("{:?}", module);
    assert!(debug.contains("ConditionalComponentModule"));
    assert!(debug.contains("test"));
}

#[test]
fn conditional_module_populate_all_fields() {
    let mut module = ConditionalComponentModule::new("test-mod", TrueCond);
    module.register(vernal_beans::ComponentDefinition::shared_value::<i32>(1));
    module.register_all(vec![vernal_beans::ComponentDefinition::shared_value::<u32>(2)]);

    let binding = vernal_beans::TraitBinding::new::<dyn std::fmt::Display + Send + Sync, String, _>(
        |arc_string: Arc<String>| -> Arc<dyn std::fmt::Display + Send + Sync> { arc_string },
    );
    module.bind(binding);
    module.bind_all(vec![]);

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
}

// ════════════════════════════════════════════════════════════════════
// ApplicationModuleRegistrar - 更多路径
// ════════════════════════════════════════════════════════════════════

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
// ConfigurationPhase - 更多路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn configuration_phase_debug() {
    let debug1 = format!("{:?}", ConfigurationPhase::ParseConfiguration);
    let debug2 = format!("{:?}", ConfigurationPhase::RegisterBean);
    assert!(!debug1.is_empty());
    assert!(!debug2.is_empty());
}

#[test]
fn configuration_phase_clone() {
    let p1 = ConfigurationPhase::ParseConfiguration;
    let p2 = p1.clone();
    assert_eq!(p1, p2);
}

#[test]
fn configuration_phase_eq() {
    assert_eq!(ConfigurationPhase::ParseConfiguration, ConfigurationPhase::ParseConfiguration);
    assert_ne!(ConfigurationPhase::ParseConfiguration, ConfigurationPhase::RegisterBean);
}

#[test]
fn configuration_phase_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(ConfigurationPhase::ParseConfiguration);
    set.insert(ConfigurationPhase::RegisterBean);
    set.insert(ConfigurationPhase::ParseConfiguration);
    assert_eq!(set.len(), 2);
}

// ════════════════════════════════════════════════════════════════════
// ScopeKey, ScopeState, TraitKey - 更多路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn scope_state_debug_all() {
    use vernal_beans::ScopeState;
    assert!(!format!("{:?}", ScopeState::Open).is_empty());
    assert!(!format!("{:?}", ScopeState::Closing).is_empty());
    assert!(!format!("{:?}", ScopeState::Closed).is_empty());
}

#[test]
fn scope_state_eq() {
    use vernal_beans::ScopeState;
    assert_eq!(ScopeState::Open, ScopeState::Open);
    assert_ne!(ScopeState::Open, ScopeState::Closed);
}

#[test]
fn scope_key_debug() {
    use vernal_beans::ScopeKey;
    let key = ScopeKey::of::<String>();
    let debug = format!("{:?}", key);
    assert!(!debug.is_empty());
}

#[test]
fn trait_key_debug() {
    use vernal_beans::TraitKey;
    let key = TraitKey::of::<dyn std::fmt::Display>();
    let debug = format!("{:?}", key);
    assert!(!debug.is_empty());
}

// ════════════════════════════════════════════════════════════════════
// OperationMetadata - 更多路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn operation_metadata_empty_debug() {
    use vernal_aop::OperationMetadata;
    let meta = OperationMetadata::empty();
    let debug = format!("{:?}", meta);
    assert!(!debug.is_empty());
}

#[test]
fn operation_metadata_with_tag_debug() {
    use vernal_aop::OperationMetadata;
    let meta = OperationMetadata::empty().with_tag("tag").unwrap();
    let debug = format!("{:?}", meta);
    assert!(!debug.is_empty());
}

#[test]
fn operation_metadata_with_qualifier_debug() {
    use vernal_aop::OperationMetadata;
    let meta = OperationMetadata::empty().with_qualifier("qual").unwrap();
    let debug = format!("{:?}", meta);
    assert!(!debug.is_empty());
}

#[test]
fn operation_metadata_eq() {
    use vernal_aop::OperationMetadata;
    let m1 = OperationMetadata::empty();
    let m2 = OperationMetadata::empty();
    assert_eq!(m1, m2);
}

// ════════════════════════════════════════════════════════════════════
// ApplicationRunnerFailure - 更多路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn application_runner_failure_display() {
    let err = ApplicationRunnerFailure::new("runner", Arc::new(std::io::Error::other("fail")) as SharedError);
    let display = format!("{err}");
    assert!(display.contains("runner"));
    assert!(display.contains("failed"));
}

#[test]
fn application_runner_failure_debug_redacted() {
    let err = ApplicationRunnerFailure::new("runner", Arc::new(std::io::Error::other("fail")) as SharedError);
    let debug = format!("{err:?}");
    assert!(debug.contains("redacted"));
}

#[test]
fn application_runner_failure_source() {
    let err = ApplicationRunnerFailure::new("runner", Arc::new(std::io::Error::other("fail")) as SharedError);
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn application_runner_failure_runner() {
    let err = ApplicationRunnerFailure::new("runner", Arc::new(std::io::Error::other("fail")) as SharedError);
    assert_eq!(err.runner(), "runner");
}

#[test]
fn application_runner_failure_clone() {
    let err = ApplicationRunnerFailure::new("runner", Arc::new(std::io::Error::other("fail")) as SharedError);
    let cloned = err.clone();
    assert_eq!(cloned.runner(), err.runner());
}

// ════════════════════════════════════════════════════════════════════
// ConditionError - 更多路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn condition_error_all_variants_display() {
    use vernal_context::ConditionError;

    let errs = vec![
        format!("{}", ConditionError::InvalidModuleName { name: "bad" }),
        format!("{}", ConditionError::DuplicateModule { name: "m" }),
        format!("{}", ConditionError::EmptyModule { name: "e" }),
        format!("{}", ConditionError::InvalidConditionName { module: "m", condition: "c" }),
        format!("{}", ConditionError::EmptyProfileSet),
        format!("{}", ConditionError::InvalidProfile { profile: "p".into() }),
        format!("{}", ConditionError::InvalidPropertyKey { key: "k".into() }),
        format!("{}", ConditionError::EvaluationFailed {
            module: "m",
            condition: "c",
            source: Box::new(std::io::Error::other("fail")),
        }),
    ];
    for d in errs {
        assert!(!d.is_empty());
    }
}

#[test]
fn condition_error_debug_all_variants() {
    use vernal_context::ConditionError;

    let errs = vec![
        format!("{:?}", ConditionError::InvalidModuleName { name: "bad" }),
        format!("{:?}", ConditionError::DuplicateModule { name: "m" }),
        format!("{:?}", ConditionError::EmptyModule { name: "e" }),
        format!("{:?}", ConditionError::InvalidConditionName { module: "m", condition: "c" }),
        format!("{:?}", ConditionError::EmptyProfileSet),
        format!("{:?}", ConditionError::InvalidProfile { profile: "p".into() }),
        format!("{:?}", ConditionError::InvalidPropertyKey { key: "k".into() }),
        format!("{:?}", ConditionError::EvaluationFailed {
            module: "m",
            condition: "c",
            source: Box::new(std::io::Error::other("fail")),
        }),
    ];
    for d in errs {
        assert!(!d.is_empty());
    }
}

// ════════════════════════════════════════════════════════════════════
// ConfigurationPropertiesError - 更多路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn configuration_properties_error_all_environments() {
    use vernal_context::{ConfigurationPropertiesError, EnvironmentError};

    let sources = vec![
        EnvironmentError::InvalidPropertySourceName { name: "n".into() },
        EnvironmentError::DuplicatePropertySource { name: "n".into() },
        EnvironmentError::InvalidPropertyKey { key: "k".into() },
        EnvironmentError::DuplicatePropertyKey { source_name: "s".into(), key: "k".into() },
        EnvironmentError::InvalidProfile { profile: "p".into() },
        EnvironmentError::MissingProperty { key: "k".into() },
        EnvironmentError::InvalidPropertyValue { key: "k".into(), source_name: "s".into(), target_type: "T" },
        EnvironmentError::MalformedPlaceholder { property: "p".into() },
        EnvironmentError::UnresolvedPlaceholder { property: "p".into(), placeholder: "ph".into() },
        EnvironmentError::CircularPlaceholder { path: vec!["a".into()] },
        EnvironmentError::PlaceholderDepthExceeded { property: "p".into(), limit: 32 },
    ];

    for (i, source) in sources.into_iter().enumerate() {
        let err = ConfigurationPropertiesError::environment::<String>(
            "field",
            format!("key.{i}"),
            source,
        );
        let display = format!("{err}");
        assert!(!display.is_empty());
        let debug = format!("{err:?}");
        assert!(!debug.is_empty());
        assert!(std::error::Error::source(&err).is_some());
    }
}

#[test]
fn configuration_properties_error_nested_chain() {
    use vernal_context::{ConfigurationPropertiesError, EnvironmentError};

    let inner = ConfigurationPropertiesError::environment::<String>(
        "inner",
        "test.inner".to_string(),
        EnvironmentError::InvalidPropertyKey { key: "k".to_string() },
    );
    let outer = ConfigurationPropertiesError::nested::<String>(
        "outer",
        "test.outer".to_string(),
        inner,
    );
    let display = format!("{outer}");
    assert!(display.contains("outer"));
    assert!(display.contains("test.outer"));
    assert!(std::error::Error::source(&outer).is_some());
}
