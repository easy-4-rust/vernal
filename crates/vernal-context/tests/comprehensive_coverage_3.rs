//! 全面覆盖测试 - Part 3: 针对低覆盖率文件的测试
//!
//! 覆盖 application_module_registrar, conditional_component_module,
//! context_error, condition_error 等文件的未覆盖路径。

use std::any::Any;
use std::sync::Arc;
use std::time::Duration;

use vernal_context::{
    ApplicationEnvironment, ApplicationModuleRegistrar,
    ConditionalComponentModule, ComponentCondition,
    ConfigurationPhase, ContextError, ContextState,
    LifecyclePhase, ManagedTaskError,
};

// ════════════════════════════════════════════════════════════════════
// ApplicationModuleRegistrar - 覆盖更多路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn registrar_register_all() {
    let mut reg = ApplicationModuleRegistrar::new();
    let defs = vec![
        vernal_beans::ComponentDefinition::shared_value::<i32>(1),
        vernal_beans::ComponentDefinition::shared_value::<u32>(2),
    ];
    reg.register_all(defs);
}

#[test]
fn registrar_bind() {
    let mut reg = ApplicationModuleRegistrar::new();
    let binding = vernal_beans::TraitBinding::new::<dyn std::fmt::Display + Send + Sync, String, _>(
        |arc_string: Arc<String>| -> Arc<dyn std::fmt::Display + Send + Sync> { arc_string },
    );
    reg.bind(binding);
}

#[test]
fn registrar_bind_all() {
    let mut reg = ApplicationModuleRegistrar::new();
    let binding = vernal_beans::TraitBinding::new::<dyn std::fmt::Display + Send + Sync, String, _>(
        |arc_string: Arc<String>| -> Arc<dyn std::fmt::Display + Send + Sync> { arc_string },
    );
    reg.bind_all(vec![binding]);
}

#[test]
fn registrar_conditional() {
    struct TrueCond;
    impl ComponentCondition for TrueCond {
        fn name(&self) -> &'static str { "true" }
        fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
            Ok(true)
        }
    }
    let mut reg = ApplicationModuleRegistrar::new();
    let module = ConditionalComponentModule::new("test-mod", TrueCond);
    reg.conditional(module);
}

#[test]
fn registrar_conditionals() {
    struct TrueCond;
    impl ComponentCondition for TrueCond {
        fn name(&self) -> &'static str { "true" }
        fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
            Ok(true)
        }
    }
    let mut reg = ApplicationModuleRegistrar::new();
    let module = ConditionalComponentModule::new("test-mod", TrueCond);
    reg.conditionals(vec![module]);
}

#[test]
fn registrar_active_profile() {
    let mut reg = ApplicationModuleRegistrar::new();
    reg.active_profile("prod");
}

#[test]
fn registrar_default_profile() {
    let mut reg = ApplicationModuleRegistrar::new();
    reg.default_profile("default");
}

#[test]
fn registrar_property_source_first() {
    let mut reg = ApplicationModuleRegistrar::new();
    let source = vernal_context::MapPropertySource::new("app", [("k", "v")]).unwrap();
    reg.property_source_first(Arc::new(source));
}

#[test]
fn registrar_property_source_last() {
    let mut reg = ApplicationModuleRegistrar::new();
    let source = vernal_context::MapPropertySource::new("app", [("k", "v")]).unwrap();
    reg.property_source_last(Arc::new(source));
}

#[test]
fn registrar_operations() {
    use vernal_aop::Operation;
    let mut reg = ApplicationModuleRegistrar::new();
    let op = Operation::new("comp", "method");
    reg.operation(op);
}

#[test]
fn registrar_operations_batch() {
    use vernal_aop::Operation;
    let mut reg = ApplicationModuleRegistrar::new();
    let ops = vec![
        Operation::new("comp", "m1"),
        Operation::new("comp", "m2"),
    ];
    reg.operations(ops);
}

// ════════════════════════════════════════════════════════════════════
// ConditionalComponentModule - 更多路径
// ════════════════════════════════════════════════════════════════════

struct FailCondition;
impl ComponentCondition for FailCondition {
    fn name(&self) -> &'static str { "fail" }
    fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Err(Box::new(std::io::Error::other("fail")))
    }
}

struct TrueCond;
impl ComponentCondition for TrueCond {
    fn name(&self) -> &'static str { "true" }
    fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(true)
    }
}

struct FalseCond;
impl ComponentCondition for FalseCond {
    fn name(&self) -> &'static str { "false" }
    fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(false)
    }
}

#[test]
fn conditional_module_new_shared() {
    let cond = Arc::new(FailCondition);
    let _module = ConditionalComponentModule::shared("test", cond);
}

#[test]
fn conditional_module_with_condition_arc() {
    let cond = Arc::new(TrueCond);
    let module = ConditionalComponentModule::shared("test", cond);
    let debug = format!("{:?}", module);
    assert!(debug.contains("ConditionalComponentModule"));
    assert!(debug.contains("test"));
}

#[test]
fn conditional_module_debug_output() {
    let module = ConditionalComponentModule::new("test", FailCondition);
    let debug = format!("{:?}", module);
    assert!(debug.contains("ConditionalComponentModule"));
    assert!(debug.contains("test"));
}

#[test]
fn conditional_module_condition_name() {
    let module = ConditionalComponentModule::new("my-module", FailCondition);
    assert_eq!(module.condition_name(), "fail");
}

#[test]
fn conditional_module_condition_name_true() {
    let module = ConditionalComponentModule::new("my-module", TrueCond);
    assert_eq!(module.condition_name(), "true");
}

#[test]
fn conditional_module_name() {
    let module = ConditionalComponentModule::new("named-mod", FailCondition);
    assert_eq!(module.name(), "named-mod");
}

#[test]
fn conditional_module_phase_default() {
    let module = ConditionalComponentModule::new("test", FailCondition);
    assert_eq!(module.phase(), ConfigurationPhase::ParseConfiguration);
}

#[test]
fn conditional_module_with_phase() {
    let module = ConditionalComponentModule::new("test", FailCondition)
        .with_phase(ConfigurationPhase::RegisterBean);
    assert_eq!(module.phase(), ConfigurationPhase::RegisterBean);
}

#[test]
fn conditional_module_register_component() {
    let mut module = ConditionalComponentModule::new("test", FailCondition);
    module.register(vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
}

#[test]
fn conditional_module_register_all() {
    let mut module = ConditionalComponentModule::new("test", FailCondition);
    let defs = vec![
        vernal_beans::ComponentDefinition::shared_value::<i32>(1),
        vernal_beans::ComponentDefinition::shared_value::<u32>(2),
    ];
    module.register_all(defs);
}

#[test]
fn conditional_module_bind() {
    let mut module = ConditionalComponentModule::new("test", FailCondition);
    let binding = vernal_beans::TraitBinding::new::<dyn std::fmt::Display + Send + Sync, String, _>(
        |arc_string: Arc<String>| -> Arc<dyn std::fmt::Display + Send + Sync> { arc_string },
    );
    module.bind(binding);
}

#[test]
fn conditional_module_bind_all() {
    let mut module = ConditionalComponentModule::new("test", FailCondition);
    let binding = vernal_beans::TraitBinding::new::<dyn std::fmt::Display + Send + Sync, String, _>(
        |arc_string: Arc<String>| -> Arc<dyn std::fmt::Display + Send + Sync> { arc_string },
    );
    module.bind_all(vec![binding]);
}

#[test]
fn conditional_module_lifecycle() {
    struct TestLifecycle;
    impl vernal_context::Lifecycle for TestLifecycle {}
    let mut module = ConditionalComponentModule::new("test", FailCondition);
    module.lifecycle::<TestLifecycle>();
}

#[test]
fn conditional_module_event_listener() {
    struct MyEvent;
    struct MyListener;
    impl vernal_context::ApplicationEventListener<MyEvent> for MyListener {
        type Error = std::io::Error;
        async fn on_event(&self, _: Arc<MyEvent>) -> Result<(), Self::Error> { Ok(()) }
    }
    let mut module = ConditionalComponentModule::new("test", FailCondition);
    module.event_listener::<MyEvent, MyListener>();
}

#[test]
fn conditional_module_application_runner() {
    struct MyRunner;
    impl vernal_context::ApplicationRunner for MyRunner {
        type Error = std::io::Error;
        async fn run(&self, _: tokio_util::sync::CancellationToken) -> Result<(), Self::Error> { Ok(()) }
    }
    let mut module = ConditionalComponentModule::new("test", FailCondition);
    module.application_runner::<MyRunner>();
}

#[test]
fn conditional_module_scheduled_task() {
    struct MyTask;
    impl vernal_context::ScheduledTask for MyTask {
        type Error = std::io::Error;
        fn schedule(&self) -> vernal_context::TaskSchedule {
            vernal_context::TaskSchedule::fixed_rate(Duration::from_secs(60)).unwrap()
        }
        async fn run(&self, _: tokio_util::sync::CancellationToken) -> Result<(), Self::Error> { Ok(()) }
    }
    let mut module = ConditionalComponentModule::new("test", FailCondition);
    module.scheduled_task::<MyTask>();
}

#[test]
fn conditional_module_configuration_properties() {
    struct TestConfig;
    impl vernal_context::ConfigurationProperties for TestConfig {
        const PREFIX: &'static str = "test";
        fn bind_with_prefix(_env: &ApplicationEnvironment, _prefix: &str) -> Result<Self, vernal_context::ConfigurationPropertiesError> {
            Ok(TestConfig)
        }
    }
    let mut module = ConditionalComponentModule::new("test", FailCondition);
    module.configuration_properties::<TestConfig>();
}

// ════════════════════════════════════════════════════════════════════
// ContextError - 覆盖更多 Display 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn context_error_invalid_state_display() {
    let err = ContextError::InvalidState {
        operation: "refresh",
        state: ContextState::Created,
    };
    let display = format!("{err}");
    assert!(display.contains("refresh"));
}

#[test]
fn context_error_invalid_state_debug() {
    let err = ContextError::InvalidState {
        operation: "refresh",
        state: ContextState::Created,
    };
    let debug = format!("{err:?}");
    assert!(debug.contains("InvalidState"));
}

#[test]
fn context_error_lifecycle_timeout_display() {
    let err = ContextError::LifecycleTimeout {
        component: "db",
        phase: LifecyclePhase::Stop,
        timeout: Duration::from_secs(30),
        abort_settled: false,
    };
    let display = format!("{err}");
    assert!(display.contains("db"));
}

#[test]
fn context_error_lifecycle_timeout_debug() {
    let err = ContextError::LifecycleTimeout {
        component: "db",
        phase: LifecyclePhase::Stop,
        timeout: Duration::from_secs(30),
        abort_settled: true,
    };
    let debug = format!("{err:?}");
    assert!(debug.contains("LifecycleTimeout"));
}

#[test]
fn context_error_lifecycle_coordinator_display() {
    let err = ContextError::LifecycleCoordinator {
        operation: "refresh",
        source: Arc::new(std::io::Error::other("test")) as vernal_core::SharedError,
    };
    let display = format!("{err}");
    assert!(display.contains("refresh"));
}

#[test]
fn context_error_lifecycle_coordinator_debug() {
    let err = ContextError::LifecycleCoordinator {
        operation: "refresh",
        source: Arc::new(std::io::Error::other("test")) as vernal_core::SharedError,
    };
    let debug = format!("{err:?}");
    assert!(debug.contains("LifecycleCoordinator"));
}

#[test]
fn context_error_lifecycle_cancelled_display() {
    let err = ContextError::LifecycleCancelled { operation: "start" };
    let display = format!("{err}");
    assert!(display.contains("start"));
}

#[test]
fn context_error_lifecycle_cancelled_debug() {
    let err = ContextError::LifecycleCancelled { operation: "start" };
    let debug = format!("{err:?}");
    assert!(debug.contains("LifecycleCancelled"));
}

#[test]
fn context_error_lifecycle_display() {
    let err = ContextError::Lifecycle {
        component: "db",
        phase: LifecyclePhase::Start,
        source: Arc::new(std::io::Error::other("fail")) as vernal_core::SharedError,
    };
    let display = format!("{err}");
    assert!(display.contains("db"));
}

#[test]
fn context_error_lifecycle_debug() {
    let err = ContextError::Lifecycle {
        component: "db",
        phase: LifecyclePhase::Start,
        source: Arc::new(std::io::Error::other("fail")) as vernal_core::SharedError,
    };
    let debug = format!("{err:?}");
    assert!(debug.contains("Lifecycle"));
}

#[test]
fn context_error_managed_task_display() {
    let err = ContextError::ManagedTask { source: ManagedTaskError::InvalidName };
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn context_error_managed_task_debug() {
    let err = ContextError::ManagedTask { source: ManagedTaskError::InvalidName };
    let debug = format!("{err:?}");
    assert!(debug.contains("ManagedTask"));
}

#[test]
fn context_error_shutdown_signal_display() {
    let err = ContextError::ShutdownSignal {
        source: Arc::new(std::io::Error::other("sig")) as vernal_core::SharedError,
    };
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn context_error_shutdown_signal_debug() {
    let err = ContextError::ShutdownSignal {
        source: Arc::new(std::io::Error::other("sig")) as vernal_core::SharedError,
    };
    let debug = format!("{err:?}");
    assert!(debug.contains("ShutdownSignal"));
}

#[test]
fn context_error_pause_restart_display() {
    let err = ContextError::PauseRestart {
        operation: "pause",
        component: "c",
        phase: LifecyclePhase::Pause,
        source: Arc::new(std::io::Error::other("fail")) as vernal_core::SharedError,
    };
    let display = format!("{err}");
    assert!(display.contains("pause"));
}

#[test]
fn context_error_pause_restart_debug() {
    let err = ContextError::PauseRestart {
        operation: "pause",
        component: "c",
        phase: LifecyclePhase::Pause,
        source: Arc::new(std::io::Error::other("fail")) as vernal_core::SharedError,
    };
    let debug = format!("{err:?}");
    assert!(debug.contains("PauseRestart"));
}

#[test]
fn context_error_lifecycle_definition_not_found() {
    use vernal_beans::ComponentKey;
    let err = ContextError::LifecycleDefinitionNotFound {
        component: ComponentKey::of::<String>(),
    };
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn context_error_runner_definition_not_found() {
    use vernal_beans::ComponentKey;
    let err = ContextError::ApplicationRunnerDefinitionNotFound {
        component: ComponentKey::of::<String>(),
    };
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn context_error_runner_scope() {
    use vernal_beans::ComponentKey;
    let err = ContextError::ApplicationRunnerScope {
        component: ComponentKey::of::<String>(),
        scope: "singleton",
    };
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn context_error_duplicate_runner() {
    use vernal_beans::ComponentKey;
    let err = ContextError::DuplicateApplicationRunner {
        component: ComponentKey::of::<String>(),
    };
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn context_error_scheduled_task_definition_not_found() {
    use vernal_beans::ComponentKey;
    let err = ContextError::ScheduledTaskDefinitionNotFound {
        component: ComponentKey::of::<String>(),
    };
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn context_error_scheduled_task_scope() {
    use vernal_beans::ComponentKey;
    let err = ContextError::ScheduledTaskScope {
        component: ComponentKey::of::<String>(),
        scope: "singleton",
    };
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn context_error_duplicate_scheduled_task() {
    use vernal_beans::ComponentKey;
    let err = ContextError::DuplicateScheduledTask {
        component: ComponentKey::of::<String>(),
    };
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn context_error_event_listener_definition_not_found() {
    use vernal_beans::ComponentKey;
    let err = ContextError::EventListenerDefinitionNotFound {
        component: ComponentKey::of::<String>(),
        event: "OrderCreated",
    };
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn context_error_event_listener_scope() {
    use vernal_beans::ComponentKey;
    let err = ContextError::EventListenerScope {
        component: ComponentKey::of::<String>(),
        event: "OrderCreated",
        scope: "singleton",
    };
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn context_error_duplicate_event_listener() {
    use vernal_beans::ComponentKey;
    let err = ContextError::DuplicateEventListener {
        component: ComponentKey::of::<String>(),
        event: "OrderCreated",
    };
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn context_error_application_runner_timeout() {
    let err = ContextError::ApplicationRunnerTimeout {
        runner: "my-runner",
        timeout: Duration::from_secs(30),
        abort_settled: false,
    };
    let display = format!("{err}");
    assert!(display.contains("my-runner"));
}

#[test]
fn context_error_application_runner_timeout_debug() {
    let err = ContextError::ApplicationRunnerTimeout {
        runner: "my-runner",
        timeout: Duration::from_secs(30),
        abort_settled: true,
    };
    let debug = format!("{err:?}");
    assert!(debug.contains("ApplicationRunnerTimeout"));
}

#[test]
fn context_error_debug_all_variants() {
    use vernal_beans::ComponentKey;
    let errs: Vec<String> = vec![
        format!("{:?}", ContextError::LifecycleDefinitionNotFound { component: ComponentKey::of::<String>() }),
        format!("{:?}", ContextError::ApplicationRunnerDefinitionNotFound { component: ComponentKey::of::<String>() }),
        format!("{:?}", ContextError::ApplicationRunnerScope { component: ComponentKey::of::<String>(), scope: "s" }),
        format!("{:?}", ContextError::DuplicateApplicationRunner { component: ComponentKey::of::<String>() }),
        format!("{:?}", ContextError::ScheduledTaskDefinitionNotFound { component: ComponentKey::of::<String>() }),
        format!("{:?}", ContextError::ScheduledTaskScope { component: ComponentKey::of::<String>(), scope: "s" }),
        format!("{:?}", ContextError::DuplicateScheduledTask { component: ComponentKey::of::<String>() }),
        format!("{:?}", ContextError::EventListenerDefinitionNotFound { component: ComponentKey::of::<String>(), event: "e" }),
        format!("{:?}", ContextError::EventListenerScope { component: ComponentKey::of::<String>(), event: "e", scope: "s" }),
        format!("{:?}", ContextError::DuplicateEventListener { component: ComponentKey::of::<String>(), event: "e" }),
    ];
    for d in errs {
        assert!(!d.is_empty());
    }
}

// ════════════════════════════════════════════════════════════════════
// ConditionError - 覆盖更多路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn condition_error_invalid_property_key_display() {
    let err = vernal_context::ConditionError::InvalidPropertyKey { key: "k".into() };
    let display = format!("{err}");
    assert!(display.contains("property"));
}

#[test]
fn condition_error_invalid_profile_display() {
    let err = vernal_context::ConditionError::InvalidProfile { profile: "p".into() };
    let display = format!("{err}");
    assert!(display.contains("profile"));
}

#[test]
fn condition_error_empty_profile_set_display() {
    let err = vernal_context::ConditionError::EmptyProfileSet;
    let display = format!("{err}");
    assert!(display.contains("profile"));
}

#[test]
fn condition_error_duplicate_module_display() {
    let err = vernal_context::ConditionError::DuplicateModule { name: "m" };
    let display = format!("{err}");
    assert!(display.contains("m"));
}

#[test]
fn condition_error_invalid_module_name_display() {
    let err = vernal_context::ConditionError::InvalidModuleName { name: "bad" };
    let display = format!("{err}");
    assert!(display.contains("invalid"));
}

#[test]
fn condition_error_empty_module_display() {
    let err = vernal_context::ConditionError::EmptyModule { name: "e" };
    let display = format!("{err}");
    assert!(display.contains("e"));
}

#[test]
fn condition_error_invalid_condition_name_display() {
    let err = vernal_context::ConditionError::InvalidConditionName { module: "m", condition: "c" };
    let display = format!("{err}");
    assert!(display.contains("m"));
}

#[test]
fn condition_error_evaluation_failed_display() {
    let err = vernal_context::ConditionError::EvaluationFailed {
        module: "m",
        condition: "c",
        source: Box::new(std::io::Error::other("fail")),
    };
    let display = format!("{err}");
    assert!(display.contains("m"));
}

#[test]
fn condition_error_debug_all() {
    let variants = vec![
        format!("{:?}", vernal_context::ConditionError::InvalidPropertyKey { key: "k".into() }),
        format!("{:?}", vernal_context::ConditionError::InvalidProfile { profile: "p".into() }),
        format!("{:?}", vernal_context::ConditionError::EmptyProfileSet),
        format!("{:?}", vernal_context::ConditionError::DuplicateModule { name: "m" }),
        format!("{:?}", vernal_context::ConditionError::InvalidModuleName { name: "bad" }),
        format!("{:?}", vernal_context::ConditionError::EmptyModule { name: "e" }),
        format!("{:?}", vernal_context::ConditionError::InvalidConditionName { module: "m", condition: "c" }),
        format!("{:?}", vernal_context::ConditionError::EvaluationFailed {
            module: "m", condition: "c",
            source: Box::new(std::io::Error::other("fail")),
        }),
    ];
    for d in variants {
        assert!(!d.is_empty());
    }
}

#[test]
fn condition_error_source_chain() {
    let err = vernal_context::ConditionError::EvaluationFailed {
        module: "m",
        condition: "c",
        source: Box::new(std::io::Error::other("fail")),
    };
    assert!(std::error::Error::source(&err).is_some());

    // Other variants should have no source
    let err2 = vernal_context::ConditionError::InvalidPropertyKey { key: "k".into() };
    assert!(std::error::Error::source(&err2).is_none());

    let err3 = vernal_context::ConditionError::InvalidProfile { profile: "p".into() };
    assert!(std::error::Error::source(&err3).is_none());

    let err4 = vernal_context::ConditionError::EmptyProfileSet;
    assert!(std::error::Error::source(&err4).is_none());

    let err5 = vernal_context::ConditionError::DuplicateModule { name: "m" };
    assert!(std::error::Error::source(&err5).is_none());

    let err6 = vernal_context::ConditionError::InvalidModuleName { name: "bad" };
    assert!(std::error::Error::source(&err6).is_none());

    let err7 = vernal_context::ConditionError::EmptyModule { name: "e" };
    assert!(std::error::Error::source(&err7).is_none());

    let err8 = vernal_context::ConditionError::InvalidConditionName { module: "m", condition: "c" };
    assert!(std::error::Error::source(&err8).is_none());
}

// ════════════════════════════════════════════════════════════════════
// EnvironmentError - 覆盖更多 Display 路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn environment_error_all_display() {
    let variants = vec![
        format!("{}", vernal_context::EnvironmentError::InvalidPropertySourceName { name: "n".into() }),
        format!("{}", vernal_context::EnvironmentError::DuplicatePropertySource { name: "n".into() }),
        format!("{}", vernal_context::EnvironmentError::InvalidPropertyKey { key: "k".into() }),
        format!("{}", vernal_context::EnvironmentError::DuplicatePropertyKey { source_name: "s".into(), key: "k".into() }),
        format!("{}", vernal_context::EnvironmentError::InvalidProfile { profile: "p".into() }),
        format!("{}", vernal_context::EnvironmentError::MissingProperty { key: "k".into() }),
        format!("{}", vernal_context::EnvironmentError::InvalidPropertyValue {
            key: "k".into(), source_name: "s".into(), target_type: "String",
        }),
        format!("{}", vernal_context::EnvironmentError::MalformedPlaceholder { property: "p".into() }),
        format!("{}", vernal_context::EnvironmentError::UnresolvedPlaceholder {
            property: "p".into(), placeholder: "ph".into(),
        }),
        format!("{}", vernal_context::EnvironmentError::CircularPlaceholder {
            path: vec!["a".into(), "b".into()],
        }),
        format!("{}", vernal_context::EnvironmentError::PlaceholderDepthExceeded {
            property: "p".into(), limit: 32,
        }),
    ];
    for d in variants {
        assert!(!d.is_empty());
    }
}

#[test]
fn environment_error_debug_all() {
    let variants = vec![
        format!("{:?}", vernal_context::EnvironmentError::InvalidPropertySourceName { name: "n".into() }),
        format!("{:?}", vernal_context::EnvironmentError::DuplicatePropertySource { name: "n".into() }),
        format!("{:?}", vernal_context::EnvironmentError::InvalidPropertyKey { key: "k".into() }),
        format!("{:?}", vernal_context::EnvironmentError::DuplicatePropertyKey { source_name: "s".into(), key: "k".into() }),
        format!("{:?}", vernal_context::EnvironmentError::InvalidProfile { profile: "p".into() }),
        format!("{:?}", vernal_context::EnvironmentError::MissingProperty { key: "k".into() }),
        format!("{:?}", vernal_context::EnvironmentError::InvalidPropertyValue {
            key: "k".into(), source_name: "s".into(), target_type: "String",
        }),
        format!("{:?}", vernal_context::EnvironmentError::MalformedPlaceholder { property: "p".into() }),
        format!("{:?}", vernal_context::EnvironmentError::UnresolvedPlaceholder {
            property: "p".into(), placeholder: "ph".into(),
        }),
        format!("{:?}", vernal_context::EnvironmentError::CircularPlaceholder {
            path: vec!["a".into(), "b".into()],
        }),
        format!("{:?}", vernal_context::EnvironmentError::PlaceholderDepthExceeded {
            property: "p".into(), limit: 32,
        }),
    ];
    for d in variants {
        assert!(!d.is_empty());
    }
}

#[test]
fn environment_error_source_chain() {
    use vernal_core::SharedError;
    let err = vernal_context::EnvironmentError::PropertySource {
        source_name: "s".into(),
        source: Arc::new(std::io::Error::other("fail")) as SharedError,
    };
    assert!(std::error::Error::source(&err).is_some());

    // Other variants should have no source
    let err2 = vernal_context::EnvironmentError::InvalidPropertySourceName { name: "n".into() };
    assert!(std::error::Error::source(&err2).is_none());

    let err3 = vernal_context::EnvironmentError::MissingProperty { key: "k".into() };
    assert!(std::error::Error::source(&err3).is_none());
}

// ════════════════════════════════════════════════════════════════════
// ConfigurationPropertiesError
// ════════════════════════════════════════════════════════════════════

#[test]
fn configuration_properties_error_display() {
    use vernal_context::ConfigurationPropertiesError;
    use vernal_context::EnvironmentError;

    let err = ConfigurationPropertiesError::environment::<String>(
        "name",
        "test.name".to_string(),
        EnvironmentError::InvalidPropertyKey { key: "k".to_string() },
    );
    let display = format!("{err}");
    assert!(display.contains("name"));
    assert!(display.contains("test.name"));
}

#[test]
fn configuration_properties_error_debug() {
    use vernal_context::ConfigurationPropertiesError;
    use vernal_context::EnvironmentError;

    let err = ConfigurationPropertiesError::environment::<String>(
        "name",
        "test.name".to_string(),
        EnvironmentError::InvalidPropertyKey { key: "k".to_string() },
    );
    let debug = format!("{:?}", err);
    assert!(debug.contains("ConfigurationPropertiesError"));
    assert!(debug.contains("redacted"));
}

#[test]
fn configuration_properties_error_nested_display() {
    use vernal_context::ConfigurationPropertiesError;
    use vernal_context::EnvironmentError;

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
}

#[test]
fn configuration_properties_error_nested_debug() {
    use vernal_context::ConfigurationPropertiesError;
    use vernal_context::EnvironmentError;

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
    let debug = format!("{outer:?}");
    assert!(debug.contains("redacted"));
}

#[test]
fn configuration_properties_error_source() {
    use vernal_context::ConfigurationPropertiesError;
    use vernal_context::EnvironmentError;

    let err = ConfigurationPropertiesError::environment::<String>(
        "name",
        "test.name".to_string(),
        EnvironmentError::InvalidPropertyKey { key: "k".to_string() },
    );
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn configuration_properties_error_nested_source() {
    use vernal_context::ConfigurationPropertiesError;
    use vernal_context::EnvironmentError;

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
    assert!(std::error::Error::source(&outer).is_some());
}

#[test]
fn configuration_properties_error_fields() {
    use vernal_context::ConfigurationPropertiesError;
    use vernal_context::EnvironmentError;

    let err = ConfigurationPropertiesError::environment::<String>(
        "name",
        "test.name".to_string(),
        EnvironmentError::InvalidPropertyKey { key: "k".to_string() },
    );
    assert_eq!(err.field(), "name");
    assert_eq!(err.property_key(), "test.name");
    assert!(!err.configuration_type().is_empty());
}

#[test]
fn configuration_properties_error_all_environments() {
    use vernal_context::ConfigurationPropertiesError;
    use vernal_context::EnvironmentError;

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
    }
}

// ════════════════════════════════════════════════════════════════════
// ConfigurationPhase
// ════════════════════════════════════════════════════════════════════

#[test]
fn configuration_phase_parse() {
    let phase = ConfigurationPhase::ParseConfiguration;
    assert_eq!(phase.as_str(), "parse_configuration");
}

#[test]
fn configuration_phase_register() {
    let phase = ConfigurationPhase::RegisterBean;
    assert_eq!(phase.as_str(), "register_bean");
}

#[test]
fn configuration_phase_debug() {
    assert!(!format!("{:?}", ConfigurationPhase::ParseConfiguration).is_empty());
    assert!(!format!("{:?}", ConfigurationPhase::RegisterBean).is_empty());
}

#[test]
fn configuration_phase_clone() {
    let phase = ConfigurationPhase::ParseConfiguration;
    let cloned = phase.clone();
    assert_eq!(phase, cloned);
}

#[test]
fn configuration_phase_serde_roundtrip() {
    let json = serde_json::to_string(&ConfigurationPhase::ParseConfiguration).unwrap();
    assert!(json.contains("parse_configuration"));
    let json2 = serde_json::to_string(&ConfigurationPhase::RegisterBean).unwrap();
    assert!(json2.contains("register_bean"));
}

// ════════════════════════════════════════════════════════════════════
// ComponentLifecycle - 覆盖更多路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn lifecycle_trait_defaults() {
    use vernal_context::Lifecycle;

    struct TestLifecycle;
    impl Lifecycle for TestLifecycle {}

    let l = TestLifecycle;
    assert!(!l.name().is_empty());

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(l.initialize()).unwrap();
    rt.block_on(l.start(tokio_util::sync::CancellationToken::new())).unwrap();
    rt.block_on(l.stop()).unwrap();
    rt.block_on(l.pause()).unwrap();

    assert!(l.is_auto_startup());
    assert!(l.is_pauseable());
    assert_eq!(l.phase(), i32::MAX);
}

#[test]
fn lifecycle_trait_overrides() {
    use vernal_context::Lifecycle;

    struct TestLifecycle;
    impl Lifecycle for TestLifecycle {
        fn name(&self) -> &'static str { "custom-name" }
        fn is_auto_startup(&self) -> bool { false }
        fn is_pauseable(&self) -> bool { false }
        fn phase(&self) -> i32 { 100 }
    }

    let l = TestLifecycle;
    assert_eq!(l.name(), "custom-name");
    assert!(!l.is_auto_startup());
    assert!(!l.is_pauseable());
    assert_eq!(l.phase(), 100);
}

// ════════════════════════════════════════════════════════════════════
// ApplicationEnvironment - 覆盖更多路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn environment_property_missing() {
    let env = ApplicationEnvironment::builder().build();
    let result = env.property("missing.key");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn environment_property_with_placeholder() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new(
        "app",
        [("db.url", "jdbc:postgresql://${db.host}:${db.port}/mydb"),
         ("db.host", "localhost"),
         ("db.port", "5432")]
    ).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let url = env.property("db.url").unwrap().unwrap();
    assert_eq!(url, "jdbc:postgresql://localhost:5432/mydb");
}

#[test]
fn environment_property_missing_placeholder() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new(
        "app", [("url", "${missing}")]
    ).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let result = env.property("url");
    assert!(result.is_err());
}

#[test]
fn environment_property_circular_placeholder() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new(
        "app", [("a", "${b}"), ("b", "${a}")]
    ).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let result = env.property("a");
    assert!(result.is_err());
}

#[test]
fn environment_property_depth_exceeded() {
    let mut props = vec![("bottom".to_string(), "value".to_string())];
    for i in 0..40 {
        props.push((format!("key{}", i), format!("${{key{}}}", i + 1)));
    }
    props.push(("start".to_string(), "${key0}".to_string()));

    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new("app", props).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let result = env.property("start");
    assert!(result.is_err());
}

#[test]
fn environment_property_invalid_key() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new("app", [("k", "v")]).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    assert!(env.property("").is_err());
    assert!(env.property("has space").is_err());
}

#[test]
fn environment_property_invalid_profile() {
    let mut builder = ApplicationEnvironment::builder();
    let result = builder.active_profile("");
    assert!(result.is_err());
}

#[test]
fn environment_get_missing_property() {
    let env = ApplicationEnvironment::builder().build();
    let result = env.get::<String>("missing");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn environment_require_missing_property() {
    let env = ApplicationEnvironment::builder().build();
    let result = env.require::<String>("missing");
    assert!(result.is_err());
}

#[test]
fn environment_contains_property() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new("app", [("k", "v")]).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    assert!(env.contains_property("k").unwrap());
    assert!(!env.contains_property("missing").unwrap());
}

#[test]
fn environment_active_profiles() {
    let mut builder = ApplicationEnvironment::builder();
    builder.active_profile("prod").unwrap();
    builder.active_profile("us-east").unwrap();
    let env = builder.build();

    assert_eq!(env.active_profiles(), &["prod".to_string(), "us-east".to_string()]);
}

#[test]
fn environment_default_profiles() {
    let mut builder = ApplicationEnvironment::builder();
    builder.default_profile("default").unwrap();
    let env = builder.build();

    assert_eq!(env.default_profiles(), &["default".to_string()]);
}

#[test]
fn environment_effective_profiles_active() {
    let mut builder = ApplicationEnvironment::builder();
    builder.active_profile("prod").unwrap();
    let env = builder.build();

    let effective: Vec<String> = env.effective_profiles().map(|s| s.to_string()).collect();
    assert_eq!(effective, vec!["prod".to_string()]);
}

#[test]
fn environment_effective_profiles_default() {
    let mut builder = ApplicationEnvironment::builder();
    builder.default_profile("default").unwrap();
    let env = builder.build();

    let effective: Vec<String> = env.effective_profiles().map(|s| s.to_string()).collect();
    assert_eq!(effective, vec!["default".to_string()]);
}

#[test]
fn environment_is_profile_active() {
    let mut builder = ApplicationEnvironment::builder();
    builder.active_profile("prod").unwrap();
    let env = builder.build();

    assert!(env.is_profile_active("prod").unwrap());
    assert!(!env.is_profile_active("dev").unwrap());
}

#[test]
fn environment_is_profile_active_invalid() {
    let mut builder = ApplicationEnvironment::builder();
    builder.active_profile("prod").unwrap();
    let env = builder.build();

    assert!(env.is_profile_active("").is_err());
    assert!(env.is_profile_active("has space").is_err());
}

#[test]
fn environment_snapshot() {
    let mut builder = ApplicationEnvironment::builder();
    builder.active_profile("prod").unwrap();
    let env = builder.build();

    let snap = env.snapshot();
    let json = serde_json::to_string(&snap).unwrap();
    assert!(json.contains("prod"));
}

#[test]
fn environment_property_source_names() {
    let mut builder = ApplicationEnvironment::builder();
    let source1 = vernal_context::MapPropertySource::new("src1", [("k1", "v1")]).unwrap();
    let source2 = vernal_context::MapPropertySource::new("src2", [("k2", "v2")]).unwrap();
    builder.add_last(Arc::new(source1)).unwrap();
    builder.add_last(Arc::new(source2)).unwrap();
    let env = builder.build();

    let names: Vec<&str> = env.property_source_names().collect();
    assert_eq!(names, vec!["src1", "src2"]);
}

#[test]
fn environment_placeholder_default_value() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new(
        "app", [("url", "${missing.host:localhost}")]
    ).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let url = env.property("url").unwrap().unwrap();
    assert_eq!(url, "localhost");
}

#[test]
fn environment_placeholder_nested() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new(
        "app",
        [("a", "val_a"),
         ("b", "${a}_b"),
         ("c", "${b}_c")]
    ).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let c = env.property("c").unwrap().unwrap();
    assert_eq!(c, "val_a_b_c");
}

#[test]
fn environment_placeholder_empty_body() {
    let mut builder = ApplicationEnvironment::builder();
    let source = vernal_context::MapPropertySource::new(
        "app", [("url", "${}")]
    ).unwrap();
    builder.add_last(Arc::new(source)).unwrap();
    let env = builder.build();

    let result = env.property("url");
    assert!(result.is_err());
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
fn environment_debug_output() {
    let env = ApplicationEnvironment::builder().build();
    let debug = format!("{:?}", env);
    assert!(!debug.is_empty());
}

#[test]
fn environment_default() {
    let env = ApplicationEnvironment::default();
    assert!(env.active_profiles().is_empty());
}

#[test]
fn environment_empty() {
    let env = ApplicationEnvironment::empty();
    assert!(env.active_profiles().is_empty());
    // default_profiles may contain "default" or be empty depending on implementation
    let _ = env.default_profiles();
}

#[test]
fn environment_invalid_default_profile() {
    let mut builder = ApplicationEnvironment::builder();
    let result = builder.default_profile("");
    assert!(result.is_err());
}

#[test]
fn environment_multiple_property_sources() {
    let mut builder = ApplicationEnvironment::builder();
    let source1 = vernal_context::MapPropertySource::new("low", [("k", "low_val")]).unwrap();
    let source2 = vernal_context::MapPropertySource::new("high", [("k", "high_val")]).unwrap();
    builder.add_last(Arc::new(source1)).unwrap();
    builder.add_last(Arc::new(source2)).unwrap();
    let env = builder.build();

    // First source has higher priority
    let val = env.property("k").unwrap().unwrap();
    assert_eq!(val, "low_val");
}
