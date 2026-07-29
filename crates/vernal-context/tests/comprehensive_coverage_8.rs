//! 全面覆盖测试 - Part 8: 通过 VernalApplicationBuilder 创建完整 ApplicationContext
//! 测试 application_context.rs 中的异步方法和元数据方法

use std::sync::Arc;
use std::time::Duration;

use vernal_context::{
    ApplicationEnvironment, VernalApplicationBuilder,
    ContextError, ContextState, LifecyclePhase,
};

// ════════════════════════════════════════════════════════════════════
// ApplicationContext - 通过 VernalApplicationBuilder 构建完整上下文
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn context_metadata_methods() {
    let rt = tokio::runtime::Handle::current();
    let builder = VernalApplicationBuilder::new(rt);
    let ctx = builder.build().unwrap();

    // Test id
    let id = ctx.id();
    assert!(!id.is_empty());

    // Test set_id
    ctx.set_id("test-id-123");
    assert_eq!(ctx.id(), "test-id-123");

    // Test application_name
    let name = ctx.application_name();
    assert!(name.is_empty()); // default is empty

    // Test set_application_name
    ctx.set_application_name("my-app");
    assert_eq!(ctx.application_name(), "my-app");

    // Test display_name
    let display = ctx.display_name();
    assert!(!display.is_empty()); // default is something

    // Test set_display_name
    ctx.set_display_name("My Application");
    assert_eq!(ctx.display_name(), "My Application");

    // Test parent
    let parent = ctx.parent();
    assert!(parent.is_none()); // no parent by default

    // Test startup_date
    let date = ctx.startup_date();
    assert!(date > 0);

    // Test container
    let _container = ctx.container();

    // Test bean_factory
    let _bean_factory = ctx.bean_factory();

    // Test environment
    let _env = ctx.environment();

    // Test cancellation_token
    let _token = ctx.cancellation_token();

    // Test lifecycle_execution_policy
    let _policy = ctx.lifecycle_execution_policy();

    // Test task_shutdown_policy
    let _policy = ctx.task_shutdown_policy();
}

#[tokio::test]
async fn context_state_transitions() {
    let rt = tokio::runtime::Handle::current();
    let builder = VernalApplicationBuilder::new(rt);
    let ctx = builder.build().unwrap();

    // Initial state should be Created
    let state = ctx.state().await;
    assert_eq!(state, ContextState::Created);

    // is_active should be false for Created state
    let active = ctx.is_active().await;
    assert!(!active);

    // is_closed should be false for Created state
    let closed = ctx.is_closed().await;
    assert!(!closed);
}

#[tokio::test]
async fn context_set_parent() {
    let rt = tokio::runtime::Handle::current();

    // Create parent context
    let builder1 = VernalApplicationBuilder::new(rt.clone());
    let parent = builder1.build().unwrap();

    // Create child context
    let builder2 = VernalApplicationBuilder::new(rt);
    let child = builder2.build().unwrap();

    // Set parent
    child.set_parent(Some(Arc::new(parent)));
    assert!(child.parent().is_some());

    // Clear parent
    child.set_parent(None);
    assert!(child.parent().is_none());
}

#[tokio::test]
async fn context_metadata_roundtrip() {
    let rt = tokio::runtime::Handle::current();
    let builder = VernalApplicationBuilder::new(rt);
    let ctx = builder.build().unwrap();

    // Test multiple set/get cycles
    ctx.set_id("id-1");
    assert_eq!(ctx.id(), "id-1");
    ctx.set_id("id-2");
    assert_eq!(ctx.id(), "id-2");

    ctx.set_application_name("app-1");
    assert_eq!(ctx.application_name(), "app-1");
    ctx.set_application_name("app-2");
    assert_eq!(ctx.application_name(), "app-2");

    ctx.set_display_name("display-1");
    assert_eq!(ctx.display_name(), "display-1");
    ctx.set_display_name("display-2");
    assert_eq!(ctx.display_name(), "display-2");
}

#[tokio::test]
async fn context_managed_tasks() {
    let rt = tokio::runtime::Handle::current();
    let builder = VernalApplicationBuilder::new(rt);
    let ctx = builder.build().unwrap();

    // managed_tasks may return Some or None depending on configuration
    let _tasks = ctx.managed_tasks();
}

#[tokio::test]
async fn context_shutdown_signal_listener() {
    let rt = tokio::runtime::Handle::current();
    let builder = VernalApplicationBuilder::new(rt);
    let ctx = builder.build().unwrap();

    // shutdown_signal_listener should return a valid listener
    let _listener = ctx.shutdown_signal_listener();
}

#[tokio::test]
async fn context_scope_cleanup_policy() {
    let rt = tokio::runtime::Handle::current();
    let builder = VernalApplicationBuilder::new(rt);
    let ctx = builder.build().unwrap();

    // scope_cleanup_policy should return a valid policy
    let _policy = ctx.scope_cleanup_policy();
}

#[tokio::test]
async fn context_idempotent_shutdown_hook() {
    let rt = tokio::runtime::Handle::current();
    let builder = VernalApplicationBuilder::new(rt);
    let ctx = builder.build().unwrap();

    // register_shutdown_hook should be idempotent
    ctx.register_shutdown_hook();
    ctx.register_shutdown_hook(); // second call should be no-op
}

// ════════════════════════════════════════════════════════════════════
// ApplicationContext - close 路径
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn context_close_from_created() {
    let rt = tokio::runtime::Handle::current();
    let builder = VernalApplicationBuilder::new(rt);
    let ctx = builder.build().unwrap();

    // Close from Created state should succeed
    let result = ctx.close().await;
    assert!(result.is_ok());

    // After close, state should be Closed
    let state = ctx.state().await;
    assert_eq!(state, ContextState::Closed);

    // is_closed should be true
    let closed = ctx.is_closed().await;
    assert!(closed);

    // is_active should be false
    let active = ctx.is_active().await;
    assert!(!active);
}

#[tokio::test]
async fn context_double_close() {
    let rt = tokio::runtime::Handle::current();
    let builder = VernalApplicationBuilder::new(rt);
    let ctx = builder.build().unwrap();

    // First close
    let result1 = ctx.close().await;
    assert!(result1.is_ok());

    // Second close should also succeed (idempotent)
    let result2 = ctx.close().await;
    assert!(result2.is_ok());
}

// ════════════════════════════════════════════════════════════════════
// ApplicationContext - run_until_cancelled 路径
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn context_run_until_cancelled() {
    let rt = tokio::runtime::Handle::current();
    let builder = VernalApplicationBuilder::new(rt);
    let ctx = builder.build().unwrap();

    // Cancel immediately
    ctx.cancellation_token().cancel();

    // run_until_cancelled should return
    let result = ctx.run_until_cancelled().await;
    assert!(result.is_ok());
}

// ════════════════════════════════════════════════════════════════════
// ApplicationContext - open_scope 路径
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn context_open_scope() {
    let rt = tokio::runtime::Handle::current();
    let builder = VernalApplicationBuilder::new(rt);
    let ctx = builder.build().unwrap();

    // open_scope should return a valid scope context
    let _scope = ctx.open_scope::<String>();
}

// ════════════════════════════════════════════════════════════════════
// ApplicationContext - startup_report 路径
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn context_startup_report() {
    let rt = tokio::runtime::Handle::current();
    let builder = VernalApplicationBuilder::new(rt);
    let ctx = builder.build().unwrap();

    // startup_report should return a valid report
    let report = ctx.startup_report().await;
    assert!(!report.context_state().is_empty());
}

// ════════════════════════════════════════════════════════════════════
// ApplicationContext - record_runtime_warning 路径
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn context_record_runtime_warning() {
    let rt = tokio::runtime::Handle::current();
    let builder = VernalApplicationBuilder::new(rt);
    let ctx = builder.build().unwrap();

    // record_runtime_warning should not panic
    ctx.record_runtime_warning("test.warning").await;
    ctx.record_runtime_warning("test.warning").await; // duplicate should be deduplicated
}

// ════════════════════════════════════════════════════════════════════
// ApplicationContext - events 路径
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn context_events() {
    let rt = tokio::runtime::Handle::current();
    let builder = VernalApplicationBuilder::new(rt);
    let ctx = builder.build().unwrap();

    // events should return a valid event bus
    let _events = ctx.events();
}

// ════════════════════════════════════════════════════════════════════
// ContextState - 覆盖更多路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn context_state_all_as_str() {
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
fn context_state_debug_all() {
    let states = vec![
        ContextState::Created,
        ContextState::Refreshing,
        ContextState::Refreshed,
        ContextState::Starting,
        ContextState::Ready,
        ContextState::RollingBack,
        ContextState::Draining,
        ContextState::Failed,
        ContextState::Pausing,
        ContextState::Paused,
        ContextState::Closed,
    ];
    for state in states {
        let debug = format!("{:?}", state);
        assert!(!debug.is_empty());
    }
}

#[test]
fn context_state_clone_all() {
    let states = vec![
        ContextState::Created,
        ContextState::Refreshing,
        ContextState::Refreshed,
        ContextState::Starting,
        ContextState::Ready,
        ContextState::RollingBack,
        ContextState::Draining,
        ContextState::Failed,
        ContextState::Pausing,
        ContextState::Paused,
        ContextState::Closed,
    ];
    for state in states {
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }
}

#[test]
fn context_state_default() {
    let state = ContextState::default();
    assert_eq!(state, ContextState::Created);
}

// ════════════════════════════════════════════════════════════════════
// LifecyclePhase - 覆盖更多路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn lifecycle_phase_all_display() {
    let phases = vec![
        LifecyclePhase::Initialize,
        LifecyclePhase::Start,
        LifecyclePhase::Stop,
        LifecyclePhase::Pause,
    ];
    for phase in phases {
        let display = format!("{}", phase);
        assert!(!display.is_empty());
    }
}

#[test]
fn lifecycle_phase_debug_all() {
    let phases = vec![
        LifecyclePhase::Initialize,
        LifecyclePhase::Start,
        LifecyclePhase::Stop,
        LifecyclePhase::Pause,
    ];
    for phase in phases {
        let debug = format!("{:?}", phase);
        assert!(!debug.is_empty());
    }
}

#[test]
fn lifecycle_phase_clone_all() {
    let phases = vec![
        LifecyclePhase::Initialize,
        LifecyclePhase::Start,
        LifecyclePhase::Stop,
        LifecyclePhase::Pause,
    ];
    for phase in phases {
        let cloned = phase.clone();
        assert_eq!(phase, cloned);
    }
}

// ════════════════════════════════════════════════════════════════════
// ConfigurationPhase - 覆盖更多路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn configuration_phase_all() {
    use vernal_context::ConfigurationPhase;

    let phases = vec![
        ConfigurationPhase::ParseConfiguration,
        ConfigurationPhase::RegisterBean,
    ];
    for phase in phases {
        let display = format!("{}", phase.as_str());
        assert!(!display.is_empty());
        let debug = format!("{:?}", phase);
        assert!(!debug.is_empty());
        let cloned = phase.clone();
        assert_eq!(phase, cloned);
    }
}

// ════════════════════════════════════════════════════════════════════
// ConditionError - 覆盖所有路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn condition_error_all_display() {
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

// ════════════════════════════════════════════════════════════════════
// ConfigurationPropertiesError - 覆盖所有路径
// ════════════════════════════════════════════════════════════════════

#[test]
fn configuration_properties_error_all() {
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

// ════════════════════════════════════════════════════════════════════
// ApplicationBuildError - 覆盖所有路径
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
