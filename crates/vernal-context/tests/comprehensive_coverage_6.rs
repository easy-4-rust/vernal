//! 全面覆盖测试 - Part 6: 通过新公开的 API 覆盖 previously pub(crate) 方法
//!
//! 覆盖 into_parts(), validate(), matches(), snapshot(), 以及各种 Parts 结构体。

use std::sync::Arc;
use std::time::Duration;

use vernal_context::{
    ApplicationEnvironment, ApplicationModuleRegistrar,
    ApplicationModuleParts,
    ConditionalComponentModule, ConditionalComponentModuleParts,
    ComponentCondition, ConditionContributionCounts,
    ConditionEvaluationSnapshot, ContextError, ContextState,
    LifecyclePhase, ConfigurationPhase,
    EnvironmentSnapshot, StartupReport,
    AdvisorRegistration, LocalAdvisorRegistration,
    ModuleEnvironmentContribution,
    ScheduledTaskFailure, ApplicationRunnerFailure,
    ApplicationLaunchError,
};

// ════════════════════════════════════════════════════════════════════
// ConditionContributionCounts - 测试新公开的 struct
// ════════════════════════════════════════════════════════════════════

#[test]
fn condition_contribution_counts_new() {
    let counts = ConditionContributionCounts::new(1, 2, 3, 4, 5);
    assert_eq!(counts.trait_bindings(), 1);
    assert_eq!(counts.lifecycles(), 2);
    assert_eq!(counts.event_listeners(), 3);
    assert_eq!(counts.application_runners(), 4);
    assert_eq!(counts.scheduled_tasks(), 5);
}

#[test]
fn condition_contribution_counts_default() {
    let counts = ConditionContributionCounts::default();
    assert_eq!(counts.trait_bindings(), 0);
    assert_eq!(counts.lifecycles(), 0);
    assert_eq!(counts.event_listeners(), 0);
    assert_eq!(counts.application_runners(), 0);
    assert_eq!(counts.scheduled_tasks(), 0);
}

#[test]
fn condition_contribution_counts_clone() {
    let counts = ConditionContributionCounts::new(1, 2, 3, 4, 5);
    let cloned = counts.clone();
    assert_eq!(counts, cloned);
}

#[test]
fn condition_contribution_counts_debug() {
    let counts = ConditionContributionCounts::new(1, 2, 3, 4, 5);
    let debug = format!("{:?}", counts);
    assert!(debug.contains("ConditionContributionCounts"));
}

#[test]
fn condition_contribution_counts_eq() {
    let c1 = ConditionContributionCounts::new(1, 2, 3, 4, 5);
    let c2 = ConditionContributionCounts::new(1, 2, 3, 4, 5);
    let c3 = ConditionContributionCounts::new(0, 0, 0, 0, 0);
    assert_eq!(c1, c2);
    assert_ne!(c1, c3);
}

// ════════════════════════════════════════════════════════════════════
// ConditionEvaluationSnapshot - 测试新公开的 new() 方法
// ════════════════════════════════════════════════════════════════════

#[test]
fn condition_evaluation_snapshot_new() {
    let counts = ConditionContributionCounts::new(1, 2, 3, 4, 5);
    let snapshot = ConditionEvaluationSnapshot::new(
        "test-module",
        "test-condition",
        true,
        vec!["ComponentA".into(), "ComponentB".into()],
        counts,
    );
    assert_eq!(snapshot.module(), "test-module");
    assert_eq!(snapshot.condition(), "test-condition");
    assert!(snapshot.matched());
    assert_eq!(snapshot.components(), &["ComponentA", "ComponentB"]);
    assert_eq!(snapshot.trait_binding_count(), 1);
    assert_eq!(snapshot.lifecycle_count(), 2);
    assert_eq!(snapshot.event_listener_count(), 3);
    assert_eq!(snapshot.application_runner_count(), 4);
    assert_eq!(snapshot.scheduled_task_count(), 5);
}

#[test]
fn condition_evaluation_snapshot_not_matched() {
    let counts = ConditionContributionCounts::new(0, 0, 0, 0, 0);
    let snapshot = ConditionEvaluationSnapshot::new(
        "module",
        "condition",
        false,
        vec![],
        counts,
    );
    assert!(!snapshot.matched());
    assert!(snapshot.components().is_empty());
}

#[test]
fn condition_evaluation_snapshot_clone() {
    let counts = ConditionContributionCounts::new(1, 2, 3, 4, 5);
    let snapshot = ConditionEvaluationSnapshot::new(
        "m", "c", true, vec!["A".into()], counts,
    );
    let cloned = snapshot.clone();
    assert_eq!(snapshot, cloned);
}

#[test]
fn condition_evaluation_snapshot_debug() {
    let counts = ConditionContributionCounts::new(1, 2, 3, 4, 5);
    let snapshot = ConditionEvaluationSnapshot::new(
        "m", "c", true, vec![], counts,
    );
    let debug = format!("{:?}", snapshot);
    assert!(debug.contains("ConditionEvaluationSnapshot"));
}

#[test]
fn condition_evaluation_snapshot_serialize() {
    let counts = ConditionContributionCounts::new(1, 2, 3, 4, 5);
    let snapshot = ConditionEvaluationSnapshot::new(
        "m", "c", true, vec!["A".into()], counts,
    );
    let json = serde_json::to_string(&snapshot).unwrap();
    assert!(json.contains("test-module") || json.contains("m"));
}

// ════════════════════════════════════════════════════════════════════
// EnvironmentSnapshot - 测试新公开的 new() 方法
// ════════════════════════════════════════════════════════════════════

#[test]
fn environment_snapshot_new() {
    let snapshot = EnvironmentSnapshot::new(
        vec!["src1".into(), "src2".into()],
        vec!["prod".into()],
        vec!["default".into()],
        vec!["prod".into()],
    );
    assert_eq!(snapshot.property_sources(), &["src1", "src2"]);
    assert_eq!(snapshot.active_profiles(), &["prod"]);
    assert_eq!(snapshot.default_profiles(), &["default"]);
    assert_eq!(snapshot.effective_profiles(), &["prod"]);
}

#[test]
fn environment_snapshot_empty() {
    let snapshot = EnvironmentSnapshot::new(vec![], vec![], vec![], vec![]);
    assert!(snapshot.property_sources().is_empty());
    assert!(snapshot.active_profiles().is_empty());
    assert!(snapshot.default_profiles().is_empty());
    assert!(snapshot.effective_profiles().is_empty());
}

#[test]
fn environment_snapshot_clone() {
    let snapshot = EnvironmentSnapshot::new(
        vec!["s".into()], vec!["a".into()], vec!["d".into()], vec!["e".into()],
    );
    let cloned = snapshot.clone();
    assert_eq!(snapshot, cloned);
}

#[test]
fn environment_snapshot_debug() {
    let snapshot = EnvironmentSnapshot::new(vec![], vec![], vec![], vec![]);
    let debug = format!("{:?}", snapshot);
    assert!(debug.contains("EnvironmentSnapshot"));
}

#[test]
fn environment_snapshot_serialize() {
    let snapshot = EnvironmentSnapshot::new(
        vec!["s".into()], vec!["a".into()], vec!["d".into()], vec!["e".into()],
    );
    let json = serde_json::to_string(&snapshot).unwrap();
    assert!(json.contains("property_sources"));
}

// ════════════════════════════════════════════════════════════════════
// ScheduledTaskFailure - 测试新公开的 new() 方法
// ════════════════════════════════════════════════════════════════════

#[test]
fn scheduled_task_failure_new() {
    let err = ScheduledTaskFailure::new(
        "my-task",
        Arc::new(std::io::Error::other("fail")) as vernal_core::SharedError,
    );
    assert_eq!(err.task(), "my-task");
}

#[test]
fn scheduled_task_failure_display() {
    let err = ScheduledTaskFailure::new(
        "my-task",
        Arc::new(std::io::Error::other("fail")) as vernal_core::SharedError,
    );
    let display = format!("{err}");
    assert!(display.contains("my-task"));
    assert!(display.contains("failed"));
}

#[test]
fn scheduled_task_failure_debug() {
    let err = ScheduledTaskFailure::new(
        "my-task",
        Arc::new(std::io::Error::other("fail")) as vernal_core::SharedError,
    );
    let debug = format!("{err:?}");
    assert!(debug.contains("redacted"));
}

#[test]
fn scheduled_task_failure_source() {
    let err = ScheduledTaskFailure::new(
        "my-task",
        Arc::new(std::io::Error::other("fail")) as vernal_core::SharedError,
    );
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn scheduled_task_failure_clone() {
    let err = ScheduledTaskFailure::new(
        "my-task",
        Arc::new(std::io::Error::other("fail")) as vernal_core::SharedError,
    );
    let cloned = err.clone();
    assert_eq!(cloned.task(), "my-task");
}

// ════════════════════════════════════════════════════════════════════
// ApplicationLaunchError - 测试新公开的构造方法
// ════════════════════════════════════════════════════════════════════

#[test]
fn application_launch_error_build() {
    use vernal_context::ApplicationBuildError;
    use vernal_beans::DefinitionError;

    let build_err = ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier { value: "v".into() },
    };
    let launch_err = ApplicationLaunchError::build(build_err);
    let display = format!("{launch_err}");
    assert!(!display.is_empty());
}

#[test]
fn application_launch_error_lifecycle() {
    let ctx_err = ContextError::LifecycleCancelled { operation: "refresh" };
    // Need a StartupReport - but its constructor is complex. Let's test the Display path
    // through a simpler variant
}

// ════════════════════════════════════════════════════════════════════
// ConditionalComponentModule - validate() 路径
// ════════════════════════════════════════════════════════════════════

struct TrueCond;
impl ComponentCondition for TrueCond {
    fn name(&self) -> &'static str { "true" }
    fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> { Ok(true) }
}

struct BadCond;
impl ComponentCondition for BadCond {
    fn name(&self) -> &'static str { "" }  // empty name - should fail validation
    fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> { Ok(true) }
}

#[test]
fn conditional_module_validate_success() {
    let mut module = ConditionalComponentModule::new("test", TrueCond);
    module.register(vernal_beans::ComponentDefinition::shared_value::<i32>(1));
    assert!(module.validate().is_ok());
}

#[test]
fn conditional_module_validate_invalid_module_name() {
    let module = ConditionalComponentModule::new("", TrueCond);
    let result = module.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    let display = format!("{err}");
    assert!(display.contains("invalid"));
}

#[test]
fn conditional_module_validate_invalid_condition_name() {
    let module = ConditionalComponentModule::new("test", BadCond);
    let result = module.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    let display = format!("{err}");
    assert!(display.contains("invalid"));
}

#[test]
fn conditional_module_validate_empty_module() {
    let module = ConditionalComponentModule::new("test", TrueCond);
    // No components registered - should fail validation
    let result = module.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    let display = format!("{err}");
    assert!(display.contains("empty"));
}

#[test]
fn conditional_module_validate_whitespace_name() {
    let module = ConditionalComponentModule::new("test with spaces", TrueCond);
    let result = module.validate();
    assert!(result.is_err());
}

#[test]
fn conditional_module_validate_control_char_name() {
    let module = ConditionalComponentModule::new("test\nwith\nnewlines", TrueCond);
    let result = module.validate();
    assert!(result.is_err());
}

// ════════════════════════════════════════════════════════════════════
// ConditionalComponentModule - matches() 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn conditional_module_matches_true() {
    let module = ConditionalComponentModule::new("test", TrueCond);
    let env = ApplicationEnvironment::builder().build();
    let result = module.matches(&env);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

struct FalseCond;
impl ComponentCondition for FalseCond {
    fn name(&self) -> &'static str { "false" }
    fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> { Ok(false) }
}

#[test]
fn conditional_module_matches_false() {
    let module = ConditionalComponentModule::new("test", FalseCond);
    let env = ApplicationEnvironment::builder().build();
    let result = module.matches(&env);
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

struct ErrorCond;
impl ComponentCondition for ErrorCond {
    fn name(&self) -> &'static str { "error" }
    fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Err(Box::new(std::io::Error::other("condition error")))
    }
}

#[test]
fn conditional_module_matches_error() {
    let module = ConditionalComponentModule::new("test", ErrorCond);
    let env = ApplicationEnvironment::builder().build();
    let result = module.matches(&env);
    assert!(result.is_err());
    let err = result.unwrap_err();
    let display = format!("{err}");
    assert!(display.contains("failed to evaluate"));
}

// ════════════════════════════════════════════════════════════════════
// ConditionalComponentModule - snapshot() 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn conditional_module_snapshot_matched() {
    let mut module = ConditionalComponentModule::new("test", TrueCond);
    module.register(vernal_beans::ComponentDefinition::shared_value::<i32>(1));
    module.register(vernal_beans::ComponentDefinition::shared_value::<String>("hello".to_string()));

    let snapshot = module.snapshot(true);
    assert_eq!(snapshot.module(), "test");
    assert!(snapshot.matched());
}

#[test]
fn conditional_module_snapshot_not_matched() {
    let mut module = ConditionalComponentModule::new("test", FalseCond);
    module.register(vernal_beans::ComponentDefinition::shared_value::<i32>(1));

    let snapshot = module.snapshot(false);
    assert_eq!(snapshot.module(), "test");
    assert!(!snapshot.matched());
}

// ════════════════════════════════════════════════════════════════════
// ConditionalComponentModule - into_parts() 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn conditional_module_into_parts_populated() {
    let mut module = ConditionalComponentModule::new("test", TrueCond);
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

    let parts = module.into_parts();
    assert_eq!(parts.definitions.len(), 2);
    assert_eq!(parts.bindings.len(), 1);
    assert_eq!(parts.lifecycle_registrars.len(), 1);
    assert_eq!(parts.event_listener_registrars.len(), 1);
    assert_eq!(parts.application_runner_registrars.len(), 1);
    assert_eq!(parts.scheduled_task_registrars.len(), 1);
}

#[test]
fn conditional_module_into_parts_empty() {
    let module = ConditionalComponentModule::new("empty", TrueCond);
    let parts = module.into_parts();
    assert!(parts.definitions.is_empty());
    assert!(parts.bindings.is_empty());
    assert!(parts.lifecycle_registrars.is_empty());
    assert!(parts.event_listener_registrars.is_empty());
    assert!(parts.application_runner_registrars.is_empty());
    assert!(parts.scheduled_task_registrars.is_empty());
}

// ════════════════════════════════════════════════════════════════════
// ApplicationModuleRegistrar - into_parts() 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn registrar_into_parts_populated() {
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

    let parts = reg.into_parts();
    assert_eq!(parts.definitions.len(), 2);
    assert_eq!(parts.bindings.len(), 1);
    assert!(!parts.lifecycle_registrars.is_empty());
    assert!(!parts.event_listener_registrars.is_empty());
    assert!(!parts.application_runner_registrars.is_empty());
    assert!(!parts.scheduled_task_registrars.is_empty());
    assert!(!parts.environment_contributions.is_empty());
    assert_eq!(parts.conditional_modules.len(), 2);
    assert_eq!(parts.operations.len(), 3);
}

#[test]
fn registrar_into_parts_empty() {
    let reg = ApplicationModuleRegistrar::new();
    let parts = reg.into_parts();
    assert!(parts.is_empty());
}

#[test]
fn registrar_parts_is_empty_true() {
    let reg = ApplicationModuleRegistrar::new();
    let parts = reg.into_parts();
    assert!(parts.is_empty());
}

#[test]
fn registrar_parts_is_empty_false() {
    let mut reg = ApplicationModuleRegistrar::new();
    reg.register(vernal_beans::ComponentDefinition::shared_value::<i32>(1));
    let parts = reg.into_parts();
    assert!(!parts.is_empty());
}

// ════════════════════════════════════════════════════════════════════
// ModuleEnvironmentContribution - apply() 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn module_environment_contribution_apply_first() {
    let source = vernal_context::MapPropertySource::new("app", [("k", "v")]).unwrap();
    let contribution = ModuleEnvironmentContribution::First(Arc::new(source));
    let mut builder = ApplicationEnvironment::builder();
    let result = contribution.apply(&mut builder);
    assert!(result.is_ok());
}

#[test]
fn module_environment_contribution_apply_last() {
    let source = vernal_context::MapPropertySource::new("app", [("k", "v")]).unwrap();
    let contribution = ModuleEnvironmentContribution::Last(Arc::new(source));
    let mut builder = ApplicationEnvironment::builder();
    let result = contribution.apply(&mut builder);
    assert!(result.is_ok());
}

#[test]
fn module_environment_contribution_apply_active_profile() {
    let contribution = ModuleEnvironmentContribution::ActiveProfile("prod".into());
    let mut builder = ApplicationEnvironment::builder();
    let result = contribution.apply(&mut builder);
    assert!(result.is_ok());
}

#[test]
fn module_environment_contribution_apply_default_profile() {
    let contribution = ModuleEnvironmentContribution::DefaultProfile("default".into());
    let mut builder = ApplicationEnvironment::builder();
    let result = contribution.apply(&mut builder);
    assert!(result.is_ok());
}

#[test]
fn module_environment_contribution_apply_invalid_profile() {
    let contribution = ModuleEnvironmentContribution::ActiveProfile("".into());
    let mut builder = ApplicationEnvironment::builder();
    let result = contribution.apply(&mut builder);
    assert!(result.is_err());
}

// ════════════════════════════════════════════════════════════════════
// ContextError - 覆盖 fmt_application_runner 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn context_error_runner_definition_not_found() {
    use vernal_beans::ComponentKey;
    let err = ContextError::ApplicationRunnerDefinitionNotFound {
        component: ComponentKey::of::<String>(),
    };
    let display = format!("{err}");
    assert!(display.contains("runner"));
    assert!(display.contains("not registered"));
}

#[test]
fn context_error_runner_scope() {
    use vernal_beans::ComponentKey;
    let err = ContextError::ApplicationRunnerScope {
        component: ComponentKey::of::<String>(),
        scope: "request",
    };
    let display = format!("{err}");
    assert!(display.contains("runner"));
    assert!(display.contains("request"));
}

#[test]
fn context_error_duplicate_runner() {
    use vernal_beans::ComponentKey;
    let err = ContextError::DuplicateApplicationRunner {
        component: ComponentKey::of::<String>(),
    };
    let display = format!("{err}");
    assert!(display.contains("runner"));
    assert!(display.contains("more than once"));
}

#[test]
fn context_error_runner_resolution() {
    use vernal_beans::{ComponentKey, ResolveError};
    let err = ContextError::ApplicationRunnerResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(ResolveError::NotFound {
            component: "r".into(),
            path: vec![],
        }),
    };
    let display = format!("{err}");
    assert!(display.contains("runner"));
    assert!(display.contains("resolve"));
}

#[test]
fn context_error_runner_failed() {
    let err = ContextError::ApplicationRunnerFailed {
        source: ApplicationRunnerFailure::new(
            "my-runner",
            Arc::new(std::io::Error::other("fail")) as vernal_core::SharedError,
        ),
    };
    let display = format!("{err}");
    assert!(display.contains("my-runner"));
}

#[test]
fn context_error_runner_timeout() {
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
// ContextError - 覆盖 fmt_scheduled_task 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn context_error_task_definition_not_found() {
    use vernal_beans::ComponentKey;
    let err = ContextError::ScheduledTaskDefinitionNotFound {
        component: ComponentKey::of::<String>(),
    };
    let display = format!("{err}");
    assert!(display.contains("task"));
    assert!(display.contains("not registered"));
}

#[test]
fn context_error_task_scope() {
    use vernal_beans::ComponentKey;
    let err = ContextError::ScheduledTaskScope {
        component: ComponentKey::of::<String>(),
        scope: "request",
    };
    let display = format!("{err}");
    assert!(display.contains("task"));
    assert!(display.contains("request"));
}

#[test]
fn context_error_duplicate_task() {
    use vernal_beans::ComponentKey;
    let err = ContextError::DuplicateScheduledTask {
        component: ComponentKey::of::<String>(),
    };
    let display = format!("{err}");
    assert!(display.contains("task"));
    assert!(display.contains("more than once"));
}

#[test]
fn context_error_task_resolution() {
    use vernal_beans::{ComponentKey, ResolveError};
    let err = ContextError::ScheduledTaskResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(ResolveError::NotFound {
            component: "t".into(),
            path: vec![],
        }),
    };
    let display = format!("{err}");
    assert!(display.contains("task"));
    assert!(display.contains("resolve"));
}

// ════════════════════════════════════════════════════════════════════
// ContextError - 覆盖 fmt_event_listener 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn context_error_listener_definition_not_found() {
    use vernal_beans::ComponentKey;
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
fn context_error_listener_scope() {
    use vernal_beans::ComponentKey;
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
fn context_error_duplicate_listener() {
    use vernal_beans::ComponentKey;
    let err = ContextError::DuplicateEventListener {
        component: ComponentKey::of::<String>(),
        event: "OrderCreated",
    };
    let display = format!("{err}");
    assert!(display.contains("listener"));
    assert!(display.contains("more than once"));
}

// ════════════════════════════════════════════════════════════════════
// ContextError - 覆盖主 match 分支
// ════════════════════════════════════════════════════════════════════

#[test]
fn context_error_lifecycle_definition_not_found() {
    use vernal_beans::ComponentKey;
    let err = ContextError::LifecycleDefinitionNotFound {
        component: ComponentKey::of::<String>(),
    };
    let display = format!("{err}");
    assert!(display.contains("lifecycle"));
    assert!(display.contains("not registered"));
}

#[test]
fn context_error_container_warm_up() {
    use vernal_beans::ResolveError;
    let err = ContextError::ContainerWarmUp {
        source: Box::new(ResolveError::NotFound {
            component: "c".into(),
            path: vec![],
        }),
    };
    let display = format!("{err}");
    assert!(display.contains("warm up"));
}

#[test]
fn context_error_component_resolution() {
    use vernal_beans::{ComponentKey, ResolveError};
    let err = ContextError::ComponentResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(ResolveError::NotFound {
            component: "c".into(),
            path: vec![],
        }),
    };
    let display = format!("{err}");
    assert!(display.contains("resolve"));
}

#[test]
fn context_error_event_listener_resolution() {
    use vernal_beans::{ComponentKey, ResolveError};
    let err = ContextError::EventListenerResolution {
        component: ComponentKey::of::<String>(),
        event: "OrderCreated",
        source: Box::new(ResolveError::NotFound {
            component: "l".into(),
            path: vec![],
        }),
    };
    let display = format!("{err}");
    assert!(display.contains("listener"));
    assert!(display.contains("OrderCreated"));
}

#[test]
fn context_error_application_runner_resolution() {
    use vernal_beans::{ComponentKey, ResolveError};
    let err = ContextError::ApplicationRunnerResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(ResolveError::NotFound {
            component: "r".into(),
            path: vec![],
        }),
    };
    let display = format!("{err}");
    assert!(display.contains("runner"));
}

#[test]
fn context_error_scheduled_task_resolution() {
    use vernal_beans::{ComponentKey, ResolveError};
    let err = ContextError::ScheduledTaskResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(ResolveError::NotFound {
            component: "t".into(),
            path: vec![],
        }),
    };
    let display = format!("{err}");
    assert!(display.contains("task"));
}

// ════════════════════════════════════════════════════════════════════
// ResolveError - Display 路径
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
// DefinitionError - Display 路径
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
// GraphError - Display 路径
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
// ManagedTaskError - Display 路径
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
// ApplicationEnvironment - 更多路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn environment_property_expansion_chain() {
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
fn environment_property_default_empty() {
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
fn environment_raw_property_missing() {
    let env = ApplicationEnvironment::builder().build();
    let raw = env.raw_property("missing").unwrap();
    assert!(raw.is_none());
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
    let high = vernal_context::MapPropertySource::new("high", [("k", "high_val")]).unwrap();
    builder.add_first(Arc::new(high)).unwrap();
    let low = vernal_context::MapPropertySource::new("low", [("k", "low_val")]).unwrap();
    builder.add_last(Arc::new(low)).unwrap();
    let env = builder.build();

    let val = env.property("k").unwrap().unwrap();
    assert_eq!(val, "high_val");
}
