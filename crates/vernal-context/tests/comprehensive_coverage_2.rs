//! 全面覆盖测试 - Part 2: 更多路径覆盖

use std::any::Any;
use std::sync::Arc;
use std::time::Duration;

use vernal_context::{
    ApplicationEnvironment, ConfigurationPropertiesError,
    ConditionalComponentModule, ComponentCondition, EventBus, MapPropertySource,
    PropertySource, PayloadApplicationEvent,
    TaskSchedule, TaskScheduleMode, TaskPriority, LifecycleExecutionPolicy,
    ScopeCleanupPolicy, DiagnosticPhase, DiagnosticOutcome, DiagnosticState,
    SubsystemStatus, ContextState, LifecyclePhase,
    ApplicationContextEvent, ApplicationContextEventBase,
};

// ════════════════════════════════════════════════════════════════════
// ConditionalComponentModule
// ════════════════════════════════════════════════════════════════════

struct TrueCondition;
impl ComponentCondition for TrueCondition {
    fn name(&self) -> &'static str { "always-true" }
    fn matches(&self, _: &ApplicationEnvironment) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> { Ok(true) }
}

#[test]
fn conditional_module_new_name() {
    let module = ConditionalComponentModule::new("test-mod", TrueCondition);
    assert_eq!(module.name(), "test-mod");
}

#[test]
fn conditional_module_phase_default() {
    let module = ConditionalComponentModule::new("test", TrueCondition);
    assert_eq!(module.phase(), vernal_context::ConfigurationPhase::ParseConfiguration);
}

#[test]
fn conditional_module_with_phase() {
    let module = ConditionalComponentModule::new("test", TrueCondition)
        .with_phase(vernal_context::ConfigurationPhase::RegisterBean);
    assert_eq!(module.phase(), vernal_context::ConfigurationPhase::RegisterBean);
}

#[test]
fn conditional_module_register() {
    let mut module = ConditionalComponentModule::new("test", TrueCondition);
    module.register(vernal_beans::ComponentDefinition::shared_value::<u32>(32u32));
}

#[test]
fn conditional_module_debug() {
    let module = ConditionalComponentModule::new("test", TrueCondition);
    let debug = format!("{:?}", module);
    assert!(!debug.is_empty());
}

// ════════════════════════════════════════════════════════════════════
// ConfigurationPropertiesError (basic coverage only)
// ════════════════════════════════════════════════════════════════════

#[test]
fn cpe_debug() {
    let err = ConfigurationPropertiesError::environment::<u32>("field", "k".to_string(), vernal_context::EnvironmentError::InvalidPropertyKey { key: "k".into() });
    assert!(format!("{:?}", err).contains("ConfigurationPropertiesError"));
}

#[test]
fn cpe_display() {
    let err = ConfigurationPropertiesError::environment::<u32>("field", "k".to_string(), vernal_context::EnvironmentError::InvalidPropertyKey { key: "k".into() });
    assert!(!format!("{err}").is_empty());
}

#[test]
fn cpe_source() {
    let err = ConfigurationPropertiesError::environment::<u32>("field", "k".to_string(), vernal_context::EnvironmentError::InvalidPropertyKey { key: "k".into() });
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn cpe_nested_debug() {
    let inner = ConfigurationPropertiesError::environment::<u32>("inner", "k".to_string(), vernal_context::EnvironmentError::InvalidPropertyKey { key: "k".into() });
    let outer = ConfigurationPropertiesError::nested::<u32>("outer", "k".to_string(), inner);
    assert!(format!("{:?}", outer).contains("ConfigurationPropertiesError"));
}

#[test]
fn cpe_nested_display() {
    let inner = ConfigurationPropertiesError::environment::<u32>("inner", "k".to_string(), vernal_context::EnvironmentError::InvalidPropertyKey { key: "k".into() });
    let outer = ConfigurationPropertiesError::nested::<u32>("outer", "k".to_string(), inner);
    assert!(!format!("{outer}").is_empty());
}

// ════════════════════════════════════════════════════════════════════
// ConfigurationProperties (skip - requires EnvironmentError integration)
// ════════════════════════════════════════════════════════════════════

// ════════════════════════════════════════════════════════════════════
// EventBus
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn event_bus_round_trip() {
    let bus = EventBus::new();
    let mut rx = bus.subscribe::<String>().await;
    bus.publish("hello".to_string()).await;
    let msg = rx.try_recv().expect("should receive");
    assert_eq!(*msg, "hello");
}

#[tokio::test]
async fn event_bus_payload() {
    let bus = EventBus::new();
    let mut rx = bus.subscribe::<PayloadApplicationEvent<String>>().await;
    bus.publish_payload(Arc::new("src"), "payload".to_string()).await;
    let event = rx.try_recv().expect("should receive payload event");
    assert_eq!(*event.payload(), "payload");
}

#[tokio::test]
async fn event_bus_capacity() {
    use std::num::NonZeroUsize;
    let bus = EventBus::with_capacity(NonZeroUsize::new(16).unwrap());
    assert_eq!(bus.capacity(), 16);
}

#[tokio::test]
async fn event_bus_subscriber_count() {
    let bus = EventBus::new();
    assert_eq!(bus.subscriber_count::<u32>().await, 0);
    let _rx = bus.subscribe::<u32>().await;
    assert_eq!(bus.subscriber_count::<u32>().await, 1);
}

#[tokio::test]
async fn event_bus_no_subscribers() {
    let bus = EventBus::new();
    assert_eq!(bus.publish(42u32).await, 0);
}

#[tokio::test]
async fn event_bus_subscriber_drop() {
    let bus = EventBus::new();
    { let _rx = bus.subscribe::<String>().await; }
    assert_eq!(bus.publish("gone".to_string()).await, 0);
}

#[tokio::test]
async fn event_bus_lagged() {
    let bus = EventBus::new();
    { let mut rx = bus.subscribe::<String>().await; bus.publish("a".to_string()).await; let _ = rx.try_recv(); }
    let mut rx2 = bus.subscribe::<String>().await;
    let _ = rx2.try_recv();
}

// ════════════════════════════════════════════════════════════════════
// ApplicationContextEventBase
// ════════════════════════════════════════════════════════════════════

#[test]
fn event_base_new() {
    let base = ApplicationContextEventBase::new(Arc::new("src"));
    assert!(base.timestamp() > 0);
}

#[test]
fn event_base_with_timestamp() {
    let base = ApplicationContextEventBase::with_timestamp(Arc::new("src"), 1_000_000);
    assert_eq!(base.timestamp(), 1_000_000);
}

#[test]
fn event_base_source() {
    let base = ApplicationContextEventBase::new(Arc::new("src".to_string()));
    assert_eq!(base.source().downcast_ref::<String>(), Some(&"src".to_string()));
}

#[test]
fn event_base_clone() {
    let base = ApplicationContextEventBase::new(Arc::new("src"));
    let cloned = base.clone();
    assert_eq!(base.timestamp(), cloned.timestamp());
}

#[test]
fn event_base_debug() {
    let base = ApplicationContextEventBase::new(Arc::new("src"));
    assert!(format!("{:?}", base).contains("ApplicationContextEventBase"));
}

// ════════════════════════════════════════════════════════════════════
// ApplicationContextEvent trait
// ════════════════════════════════════════════════════════════════════

#[test]
fn event_type_id_fn() {
    assert_eq!(vernal_context::event_type_id::<String>(), std::any::TypeId::of::<String>());
}

#[test]
fn event_trait_impl() {
    use vernal_context::ApplicationContextEvent;
    struct TestEvent { base: ApplicationContextEventBase }
    impl ApplicationContextEvent for TestEvent {
        fn source(&self) -> &dyn Any { self.base.source() }
        fn timestamp(&self) -> i64 { self.base.timestamp() }
    }
    let event = TestEvent { base: ApplicationContextEventBase::new(Arc::new("test-source".to_string())) };
    assert!(event.timestamp() > 0);
    assert_eq!(event.source().downcast_ref::<String>(), Some(&"test-source".to_string()));
}

// ════════════════════════════════════════════════════════════════════
// TaskSchedule / TaskScheduleMode / TaskPriority
// ════════════════════════════════════════════════════════════════════

#[test]
fn task_schedule_fixed_rate() {
    let sched = TaskSchedule::fixed_rate(Duration::from_secs(60)).expect("valid");
    assert!(format!("{:?}", sched).contains("TaskSchedule"));
}

#[test]
fn task_schedule_fixed_delay() {
    let sched = TaskSchedule::fixed_delay(Duration::from_secs(30)).expect("valid");
    assert!(format!("{:?}", sched).contains("TaskSchedule"));
}

#[test]
fn task_schedule_mode_debug() {
    assert_eq!(format!("{:?}", TaskScheduleMode::FixedRate), "FixedRate");
    assert_eq!(format!("{:?}", TaskScheduleMode::FixedDelay), "FixedDelay");
}

#[test]
fn task_priority_ordering() {
    use TaskPriority::*;
    assert!(Low < Normal);
    assert!(Normal < High);
    assert!(High < Critical);
}

#[test]
fn task_options_builder() {
    let opts = vernal_context::TaskOptions::new()
        .with_timeout(Duration::from_secs(5))
        .with_max_retries(3)
        .with_priority(TaskPriority::High);
    assert_eq!(opts.timeout, Some(Duration::from_secs(5)));
    assert_eq!(opts.max_retries, 3);
    assert_eq!(opts.priority, TaskPriority::High);
}

#[test]
fn task_options_default() {
    let opts = vernal_context::TaskOptions::default();
    assert!(opts.timeout.is_none());
    assert_eq!(opts.max_retries, 0);
    assert_eq!(opts.priority, TaskPriority::Normal);
}

// ════════════════════════════════════════════════════════════════════
// LifecycleExecutionPolicy / ScopeCleanupPolicy
// ════════════════════════════════════════════════════════════════════

#[test]
fn lifecycle_execution_policy_default() {
    assert!(!format!("{:?}", LifecycleExecutionPolicy::default()).is_empty());
}

#[test]
fn scope_cleanup_policy_default() {
    assert!(!format!("{:?}", ScopeCleanupPolicy::default()).is_empty());
}

// ════════════════════════════════════════════════════════════════════
// DiagnosticPhase / DiagnosticOutcome / DiagnosticState
// ════════════════════════════════════════════════════════════════════

#[test]
fn diagnostic_phase_debug() {
    assert!(format!("{:?}", DiagnosticPhase::Initialize).contains("Initialize"));
    assert!(format!("{:?}", DiagnosticPhase::Start).contains("Start"));
    assert!(format!("{:?}", DiagnosticPhase::Stop).contains("Stop"));
}

#[test]
fn diagnostic_outcome_debug() {
    assert!(format!("{:?}", DiagnosticOutcome::Succeeded).contains("Succeeded"));
    assert!(format!("{:?}", DiagnosticOutcome::Failed).contains("Failed"));
}

#[test]
fn diagnostic_state_as_str() {
    assert_eq!(DiagnosticState::Unknown.as_str(), "unknown");
    assert_eq!(DiagnosticState::Available.as_str(), "available");
    assert_eq!(DiagnosticState::Degraded.as_str(), "degraded");
    assert_eq!(DiagnosticState::Unavailable.as_str(), "unavailable");
}

#[test]
fn diagnostic_state_default() {
    assert_eq!(DiagnosticState::default(), DiagnosticState::Unknown);
}

// ════════════════════════════════════════════════════════════════════
// SubsystemStatus / StartupObservation
// ════════════════════════════════════════════════════════════════════

#[test]
fn subsystem_status_new() {
    let status = SubsystemStatus::new("axum", DiagnosticState::Available);
    assert_eq!(status.name(), "axum");
    assert_eq!(status.state(), DiagnosticState::Available);
}

#[test]
fn subsystem_status_debug() {
    let status = SubsystemStatus::new("axum", DiagnosticState::Available);
    assert!(format!("{:?}", status).contains("SubsystemStatus"));
}

// StartupObservation::new is private - cannot test directly from tests

// ════════════════════════════════════════════════════════════════════
// ContextState / LifecyclePhase
// ════════════════════════════════════════════════════════════════════

#[test]
fn context_state_all() {
    assert_eq!(ContextState::Created.as_str(), "created");
    assert_eq!(ContextState::Refreshing.as_str(), "refreshing");
    assert_eq!(ContextState::Refreshed.as_str(), "refreshed");
    assert_eq!(ContextState::Starting.as_str(), "starting");
    assert_eq!(ContextState::Ready.as_str(), "ready");
    assert_eq!(ContextState::RollingBack.as_str(), "rolling_back");
    assert_eq!(ContextState::Draining.as_str(), "draining");
    assert_eq!(ContextState::Failed.as_str(), "failed");
    assert_eq!(ContextState::Closed.as_str(), "closed");
}

#[test]
fn lifecycle_phase_all() {
    assert_eq!(format!("{}", LifecyclePhase::Initialize), "initialize");
    assert_eq!(format!("{}", LifecyclePhase::Start), "start");
    assert_eq!(format!("{}", LifecyclePhase::Stop), "stop");
    assert_eq!(format!("{}", LifecyclePhase::Pause), "pause");
}

// ════════════════════════════════════════════════════════════════════
// MapPropertySource
// ════════════════════════════════════════════════════════════════════

#[test]
fn map_property_source_debug() {
    let source = MapPropertySource::new("app", [("k", "v")]).expect("valid");
    assert!(!format!("{:?}", source).is_empty());
}

#[test]
fn map_property_source_as_any() {
    let source = MapPropertySource::new("app", [("k", "v")]).expect("valid");
    let _ = &source as &dyn PropertySource;
}

// ════════════════════════════════════════════════════════════════════
// ApplicationContext
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn context_environment() {
    let builder = vernal_context::ApplicationContextBuilder::new(
        vernal_beans::RegistryBuilder::new().build().expect("registry")
    );
    let ctx = builder.build().expect("build");
    let _env = ctx.environment();
}

#[tokio::test]
async fn context_container() {
    let builder = vernal_context::ApplicationContextBuilder::new(
        vernal_beans::RegistryBuilder::new().build().expect("registry")
    );
    let ctx = builder.build().expect("build");
    let _container = ctx.container();
}

#[tokio::test]
async fn context_shutdown_signal() {
    let builder = vernal_context::ApplicationContextBuilder::new(
        vernal_beans::RegistryBuilder::new().build().expect("registry")
    );
    let ctx = builder.build().expect("build");
    assert!(format!("{:?}", ctx.shutdown_signal_listener()).contains("SystemShutdownSignalListener"));
}

#[tokio::test]
async fn context_scope_cleanup_policy() {
    let builder = vernal_context::ApplicationContextBuilder::new(
        vernal_beans::RegistryBuilder::new().build().expect("registry")
    );
    let ctx = builder.build().expect("build");
    let _policy = ctx.scope_cleanup_policy();
}

#[tokio::test]
async fn context_lifecycle_execution_policy() {
    let builder = vernal_context::ApplicationContextBuilder::new(
        vernal_beans::RegistryBuilder::new().build().expect("registry")
    );
    let ctx = builder.build().expect("build");
    let _policy = ctx.lifecycle_execution_policy();
}

// ════════════════════════════════════════════════════════════════════
// Debug output coverage
// ════════════════════════════════════════════════════════════════════

#[test]
fn error_debug_coverage() {
    use vernal_beans::ComponentKey;
    let errors: Vec<Box<dyn std::fmt::Debug>> = vec![
        Box::new(vernal_context::ContextError::InvalidState { operation: "t", state: ContextState::Created }),
        Box::new(vernal_context::ContextError::LifecycleDefinitionNotFound { component: ComponentKey::of::<u32>() }),
        Box::new(vernal_context::ContextError::ApplicationRunnerDefinitionNotFound { component: ComponentKey::of::<u32>() }),
        Box::new(vernal_context::ContextError::ScheduledTaskDefinitionNotFound { component: ComponentKey::of::<u32>() }),
        Box::new(vernal_context::ContextError::EventListenerDefinitionNotFound { component: ComponentKey::of::<u32>(), event: "e" }),
        Box::new(vernal_context::ContextError::ContainerWarmUp { source: Box::new(vernal_beans::ResolveError::NotFound { component: "w".into(), path: vec![] }) }),
        Box::new(vernal_context::ContextError::ComponentResolution { component: ComponentKey::of::<u32>(), source: Box::new(vernal_beans::ResolveError::NotFound { component: "r".into(), path: vec![] }) }),
        Box::new(vernal_context::ContextError::EventListenerResolution { component: ComponentKey::of::<u32>(), event: "e", source: Box::new(vernal_beans::ResolveError::NotFound { component: "r".into(), path: vec![] }) }),
        Box::new(vernal_context::ContextError::ApplicationRunnerResolution { component: ComponentKey::of::<u32>(), source: Box::new(vernal_beans::ResolveError::NotFound { component: "r".into(), path: vec![] }) }),
        Box::new(vernal_context::ContextError::ScheduledTaskResolution { component: ComponentKey::of::<u32>(), source: Box::new(vernal_beans::ResolveError::NotFound { component: "r".into(), path: vec![] }) }),
        Box::new(vernal_context::ContextError::ApplicationRunnerTimeout { runner: "r", timeout: Duration::from_secs(5), abort_settled: true }),
        Box::new(vernal_context::ContextError::Lifecycle { component: "c", phase: LifecyclePhase::Stop, source: Arc::new(std::io::Error::other("e")) }),
        Box::new(vernal_context::ContextError::LifecycleTimeout { component: "c", phase: LifecyclePhase::Stop, timeout: Duration::from_secs(1), abort_settled: false }),
        Box::new(vernal_context::ContextError::ManagedTask { source: vernal_context::ManagedTaskError::InvalidName }),
        Box::new(vernal_context::ContextError::LifecycleCancelled { operation: "test" }),
        Box::new(vernal_context::ContextError::LifecycleCoordinator { operation: "r", source: Arc::new(std::io::Error::other("e")) }),
    ];
    for e in errors {
        assert!(!format!("{:?}", e).is_empty());
    }
}
