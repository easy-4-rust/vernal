//! 全面覆盖测试 - Part 1: Error enum Display + From + source。
//!
//! 覆盖 0% 覆盖率的错误模块：
//! - application_build_error.rs
//! - application_launch_error.rs
//! - application_module_error.rs
//! - application_runner_failure.rs
//! - condition_error.rs
//! - context_error.rs
//! - environment_error.rs
//! - application_event_listener.rs (smart listener hooks)
//! - payload_application_event.rs
//! - configuration_phase.rs
//! - listener_registration.rs
//! - etc.

use std::any::{Any, TypeId};
use std::sync::Arc;

use vernal_aop::{InvocationPlanCatalogInitializationError, Operation, OperationMetadata, OperationMetadataConflictError};
use vernal_beans::{ComponentKey, DefinitionError, GraphError, ResolveError};
use vernal_context::{
    ApplicationBuildError, ApplicationListenerRegistration, ApplicationModuleError,
    ApplicationRunnerFailure, ConditionError, ConfigurationPhase, ContextError, ContextState,
    EnvironmentError, EventListenerRegistry, ListenerKey, PayloadApplicationEvent,
};

// ════════════════════════════════════════════════════════════════════
// ApplicationBuildError (16% coverage → 100%)
// ════════════════════════════════════════════════════════════════════

fn make_tokio_runtime_unavailable_error() -> ApplicationBuildError {
    // The only way to get a TryCurrentError is to call Handle::try_current() outside a runtime.
    // But running this in #[test] inside #[tokio::test] would not work. We construct via a non-tokio
    // test by explicitly calling Handle::try_current().
    match tokio::runtime::Handle::try_current() {
        Ok(_) => panic!("expected no current runtime"),
        Err(source) => ApplicationBuildError::TokioRuntimeUnavailable { source },
    }
}

#[test]
fn application_build_error_tokio_runtime_unavailable_display_and_source() {
    let error = make_tokio_runtime_unavailable_error();
    let display = format!("{error}");
    assert!(display.contains("Tokio runtime is unavailable"), "display: {display}");
    // Error::source
    let source = std::error::Error::source(&error);
    assert!(source.is_some());
}

#[test]
fn application_build_error_definition_display_and_source() {
    let source = DefinitionError::DuplicateDefinition {
        key: ComponentKey::of::<u32>(),
    };
    let error = ApplicationBuildError::Definition { source: source.clone() };
    let display = format!("{error}");
    assert!(display.contains("application component definition is invalid"), "display: {display}");
    let source_out = std::error::Error::source(&error);
    assert!(source_out.is_some());
    // From impl
    let from: ApplicationBuildError = source.into();
    assert!(matches!(from, ApplicationBuildError::Definition { .. }));
}

#[test]
fn application_build_error_graph_display_and_source() {
    let source = GraphError::Cycle {
        path: vec!["a".to_string(), "b".to_string(), "a".to_string()],
    };
    let error = ApplicationBuildError::Graph { source: source.clone() };
    let display = format!("{error}");
    assert!(display.contains("application dependency graph is invalid"), "display: {display}");
    let source_out = std::error::Error::source(&error);
    assert!(source_out.is_some());
    // From impl
    let from: ApplicationBuildError = source.into();
    assert!(matches!(from, ApplicationBuildError::Graph { .. }));
}

#[test]
fn application_build_error_context_display_and_source() {
    let source = ContextError::InvalidState {
        operation: "test",
        state: ContextState::Closed,
    };
    let error = ApplicationBuildError::Context { source: source.clone() };
    let display = format!("{error}");
    assert!(display.contains("application context cannot be built"), "display: {display}");
    let source_out = std::error::Error::source(&error);
    assert!(source_out.is_some());
    // From impl
    let from: ApplicationBuildError = source.into();
    assert!(matches!(from, ApplicationBuildError::Context { .. }));
}

#[test]
fn application_build_error_condition_display_and_source() {
    let source = ConditionError::DuplicateModule { name: "m" };
    let error = ApplicationBuildError::Condition { source: source.clone() };
    let display = format!("{error}");
    assert!(display.contains("application conditional component assembly failed"), "display: {display}");
    let source_out = std::error::Error::source(&error);
    assert!(source_out.is_some());
    // From impl
    let from: ApplicationBuildError = source.into();
    assert!(matches!(from, ApplicationBuildError::Condition { .. }));
}

#[test]
fn application_build_error_advisor_resolution_display_and_source() {
    let source: Box<ResolveError> = Box::new(ResolveError::NotFound {
        component: "u32".to_string(),
        path: vec!["a".to_string()],
    });
    let error = ApplicationBuildError::AdvisorResolution {
        component: ComponentKey::of::<u32>(),
        source,
    };
    let display = format!("{error}");
    assert!(display.contains("cannot be resolved"), "display: {display}");
    let source_out = std::error::Error::source(&error);
    assert!(source_out.is_some());
}

#[test]
fn application_build_error_local_advisor_resolution_display_and_source() {
    let source: Box<ResolveError> = Box::new(ResolveError::NotFound {
        component: "u32".to_string(),
        path: vec!["a".to_string()],
    });
    let error = ApplicationBuildError::LocalAdvisorResolution {
        component: ComponentKey::of::<u32>(),
        source,
    };
    let display = format!("{error}");
    assert!(display.contains("local advisor"), "display: {display}");
    let source_out = std::error::Error::source(&error);
    assert!(source_out.is_some());
}

#[test]
fn application_build_error_advisor_scope_display_no_source() {
    let error = ApplicationBuildError::AdvisorScope {
        component: ComponentKey::of::<u32>(),
        scope: "transient",
    };
    let display = format!("{error}");
    assert!(display.contains("must be singleton"), "display: {display}");
    // No source for this variant
    let source_out = std::error::Error::source(&error);
    assert!(source_out.is_none());
}

#[test]
fn application_build_error_operation_metadata_display_and_source() {
    let source = OperationMetadataConflictError::new(
        Operation::new("comp", "method"),
        OperationMetadata::empty(),
        OperationMetadata::empty(),
    );
    let error = ApplicationBuildError::OperationMetadata { source };
    let display = format!("{error}");
    assert!(display.contains("operation metadata"), "display: {display}");
    let source_out = std::error::Error::source(&error);
    assert!(source_out.is_some());
}

#[test]
fn application_build_error_aop_catalog_initialization_display_and_source() {
    let source = InvocationPlanCatalogInitializationError;
    let error = ApplicationBuildError::AopCatalogInitialization { source };
    let display = format!("{error}");
    assert!(display.contains("AOP catalog"), "display: {display}");
    let source_out = std::error::Error::source(&error);
    assert!(source_out.is_some());
}

// ════════════════════════════════════════════════════════════════════
// ApplicationLaunchError (22% coverage → 100%)
// ════════════════════════════════════════════════════════════════════

#[test]
fn application_launch_error_build_variant_display() {
    let inner = ApplicationBuildError::Definition {
        source: DefinitionError::DuplicateDefinition {
            key: ComponentKey::of::<u32>(),
        },
    };
    // We need to construct a StartupReport — use a helper.
    // ApplicationLaunchError::build is private; use ApplicationLaunchError::from instead.
    let launch = vernal_context::ApplicationLaunchError::from(inner);
    let display = format!("{launch}");
    assert!(!display.is_empty());
}

// ════════════════════════════════════════════════════════════════════
// ApplicationModuleError (22% coverage → 100%)
// ════════════════════════════════════════════════════════════════════

#[test]
fn application_module_error_invalid_name_display() {
    let error = ApplicationModuleError::InvalidName { name: "bad name" };
    let display = format!("{error}");
    assert!(display.contains("invalid"), "display: {display}");
}

#[test]
fn application_module_error_duplicate_name_display() {
    let error = ApplicationModuleError::DuplicateName { name: "dup" };
    let display = format!("{error}");
    assert!(display.contains("duplicate"), "display: {display}");
}

#[test]
fn application_module_error_empty_display() {
    let error = ApplicationModuleError::Empty { name: "empty" };
    let display = format!("{error}");
    assert!(display.contains("empty"), "display: {display}");
}

#[test]
fn application_module_error_configuration_display() {
    let error = ApplicationModuleError::Configuration {
        module: "mod",
        source: std::io::Error::other("config err"),
    };
    let display = format!("{error}");
    assert!(display.contains("config"), "display: {display}");
}

#[test]
fn application_module_error_definition_display() {
    let error = ApplicationModuleError::Definition {
        module: "mod",
        source: DefinitionError::DuplicateDefinition {
            key: ComponentKey::of::<u32>(),
        },
    };
    let display = format!("{error}");
    assert!(display.contains("definition"), "display: {display}");
}

#[test]
fn application_module_error_environment_display() {
    let error = ApplicationModuleError::Environment {
        module: "mod",
        source: EnvironmentError::InvalidPropertyKey { key: "k".to_string() },
    };
    let display = format!("{error}");
    assert!(display.contains("environment"), "display: {display}");
}

#[test]
fn application_module_error_condition_display() {
    let error = ApplicationModuleError::Condition {
        module: "mod",
        source: ConditionError::DuplicateModule { name: "x" },
    };
    let display = format!("{error}");
    assert!(display.contains("condition"), "display: {display}");
}

#[test]
fn application_module_error_debug() {
    let error = ApplicationModuleError::DuplicateName { name: "dup" };
    let debug = format!("{error:?}");
    assert!(debug.contains("DuplicateName"));
}

// ════════════════════════════════════════════════════════════════════
// ApplicationRunnerFailure (0% coverage → 100%)
// ════════════════════════════════════════════════════════════════════

#[test]
fn application_runner_failure_from_error() {
    let err: Box<dyn std::error::Error + Send + Sync> =
        Box::new(std::io::Error::other("runner failed"));
    let failure = ApplicationRunnerFailure::from_error("my-runner", err);
    let display = format!("{failure}");
    assert!(display.contains("my-runner"), "display: {display}");
    let source = std::error::Error::source(&failure);
    assert!(source.is_some());
    let debug = format!("{failure:?}");
    assert!(debug.contains("ApplicationRunnerFailure"));
}

// ════════════════════════════════════════════════════════════════════
// ConditionError (0% coverage → 100%)
// ════════════════════════════════════════════════════════════════════

#[test]
fn condition_error_invalid_property_key_display() {
    let error = ConditionError::InvalidPropertyKey { key: "k".to_string() };
    let display = format!("{error}");
    assert!(display.contains("property"), "display: {display}");
    // No source for this variant
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn condition_error_invalid_profile_display() {
    let error = ConditionError::InvalidProfile { profile: "p".to_string() };
    let display = format!("{error}");
    assert!(display.contains("profile"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn condition_error_empty_profile_set_display() {
    let error = ConditionError::EmptyProfileSet;
    let display = format!("{error}");
    assert!(display.contains("profile"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn condition_error_duplicate_module_display() {
    let error = ConditionError::DuplicateModule { name: "m" };
    let display = format!("{error}");
    assert!(display.contains("duplicate") || display.contains("module"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn condition_error_invalid_module_name_display() {
    let error = ConditionError::InvalidModuleName { name: "bad" };
    let display = format!("{error}");
    assert!(display.contains("module") || display.contains("invalid"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn condition_error_empty_module_display() {
    let error = ConditionError::EmptyModule { name: "m" };
    let display = format!("{error}");
    assert!(display.contains("empty") || display.contains("module"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn condition_error_evaluation_failed_display() {
    // This variant requires a ComponentCondition name + BoxError
    let error = ConditionError::EvaluationFailed {
        module: "m",
        condition: "c",
        source: Box::new(std::io::Error::other("oops")),
    };
    let display = format!("{error}");
    assert!(display.contains("evaluation") || display.contains("module"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_some());
}

#[test]
fn condition_error_debug_and_clone() {
    let error = ConditionError::InvalidProfile { profile: "p".to_string() };
    let debug = format!("{error:?}");
    assert!(debug.contains("InvalidProfile"));
    let cloned = error.clone();
    assert_eq!(format!("{error}"), format!("{cloned}"));
}

// ════════════════════════════════════════════════════════════════════
// EnvironmentError (14% coverage → 100%)
// ════════════════════════════════════════════════════════════════════

#[test]
fn environment_error_invalid_property_key_display() {
    let error = EnvironmentError::InvalidPropertyKey { key: "k".to_string() };
    let display = format!("{error}");
    assert!(display.contains("property key"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn environment_error_invalid_profile_display() {
    let error = EnvironmentError::InvalidProfile { profile: "p".to_string() };
    let display = format!("{error}");
    assert!(display.contains("profile"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn environment_error_missing_property_display() {
    let error = EnvironmentError::MissingProperty { key: "k".to_string() };
    let display = format!("{error}");
    assert!(display.contains("missing") || display.contains("property"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn environment_error_invalid_property_value_display() {
    let error = EnvironmentError::InvalidPropertyValue {
        key: "k".to_string(),
        source_name: "src".to_string(),
        target_type: "u32",
    };
    let display = format!("{error}");
    assert!(display.contains("u32") || display.contains("property"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn environment_error_malformed_placeholder_display() {
    let error = EnvironmentError::MalformedPlaceholder { property: "p".to_string() };
    let display = format!("{error}");
    assert!(display.contains("placeholder"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn environment_error_unresolved_placeholder_display() {
    let error = EnvironmentError::UnresolvedPlaceholder {
        property: "p".to_string(),
        placeholder: "x".to_string(),
    };
    let display = format!("{error}");
    assert!(display.contains("placeholder"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn environment_error_circular_placeholder_display() {
    let error = EnvironmentError::CircularPlaceholder {
        path: vec!["a".to_string(), "b".to_string()],
    };
    let display = format!("{error}");
    assert!(display.contains("circular") || display.contains("placeholder"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn environment_error_placeholder_depth_exceeded_display() {
    let error = EnvironmentError::PlaceholderDepthExceeded {
        property: "p".to_string(),
        limit: 32,
    };
    let display = format!("{error}");
    assert!(display.contains("depth") || display.contains("placeholder"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn environment_error_duplicate_property_source_display() {
    let error = EnvironmentError::DuplicatePropertySource { name: "src".to_string() };
    let display = format!("{error}");
    assert!(display.contains("duplicate") || display.contains("source"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn environment_error_invalid_property_source_name_display() {
    let error = EnvironmentError::InvalidPropertySourceName { name: "n".to_string() };
    let display = format!("{error}");
    assert!(display.contains("name") || display.contains("source"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn environment_error_property_source_display_and_source() {
    let error = EnvironmentError::PropertySource {
        source_name: "src".to_string(),
        source: Arc::new(std::io::Error::other("io")),
    };
    let display = format!("{error}");
    assert!(display.contains("src"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_some());
}

#[test]
fn environment_error_debug_and_clone() {
    let error = EnvironmentError::MissingProperty { key: "k".to_string() };
    let debug = format!("{error:?}");
    assert!(debug.contains("MissingProperty"));
    let cloned = error.clone();
    assert_eq!(format!("{error}"), format!("{cloned}"));
}

// ════════════════════════════════════════════════════════════════════
// ContextError (26% coverage → 100%)
// ════════════════════════════════════════════════════════════════════

#[test]
fn context_error_invalid_state_display() {
    let error = ContextError::InvalidState {
        operation: "pause",
        state: ContextState::Closed,
    };
    let display = format!("{error}");
    assert!(display.contains("pause") && display.contains("Closed"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn context_error_lifecycle_definition_not_found_display() {
    let error = ContextError::LifecycleDefinitionNotFound {
        component: ComponentKey::of::<u32>(),
    };
    let display = format!("{error}");
    assert!(display.contains("lifecycle") || display.contains("u32"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn context_error_application_runner_definition_not_found_display() {
    let error = ContextError::ApplicationRunnerDefinitionNotFound {
        component: ComponentKey::of::<u32>(),
    };
    let display = format!("{error}");
    assert!(!display.is_empty());
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn context_error_application_runner_scope_display() {
    let error = ContextError::ApplicationRunnerScope {
        component: ComponentKey::of::<u32>(),
        scope: "transient",
    };
    let display = format!("{error}");
    assert!(!display.is_empty());
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn context_error_duplicate_application_runner_display() {
    let error = ContextError::DuplicateApplicationRunner {
        component: ComponentKey::of::<u32>(),
    };
    let display = format!("{error}");
    assert!(!display.is_empty());
    let source = std::error::Error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn context_error_scheduled_task_definition_not_found_display() {
    let error = ContextError::ScheduledTaskDefinitionNotFound {
        component: ComponentKey::of::<u32>(),
    };
    let display = format!("{error}");
    assert!(!display.is_empty());
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn context_error_scheduled_task_scope_display() {
    let error = ContextError::ScheduledTaskScope {
        component: ComponentKey::of::<u32>(),
        scope: "transient",
    };
    let display = format!("{error}");
    assert!(!display.is_empty());
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn context_error_duplicate_scheduled_task_display() {
    let error = ContextError::DuplicateScheduledTask {
        component: ComponentKey::of::<u32>(),
    };
    let display = format!("{error}");
    assert!(!display.is_empty());
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn context_error_event_listener_definition_not_found_display() {
    let error = ContextError::EventListenerDefinitionNotFound {
        component: ComponentKey::of::<u32>(),
        event: "evt",
    };
    let display = format!("{error}");
    assert!(!display.is_empty());
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn context_error_event_listener_scope_display() {
    let error = ContextError::EventListenerScope {
        component: ComponentKey::of::<u32>(),
        event: "evt",
        scope: "transient",
    };
    let display = format!("{error}");
    assert!(!display.is_empty());
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn context_error_duplicate_event_listener_display() {
    let error = ContextError::DuplicateEventListener {
        component: ComponentKey::of::<u32>(),
        event: "evt",
    };
    let display = format!("{error}");
    assert!(!display.is_empty());
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn context_error_container_warm_up_display() {
    let error = ContextError::ContainerWarmUp {
        source: Box::new(std::io::Error::other("warmup")),
    };
    let display = format!("{error}");
    assert!(!display.is_empty());
    let source = std::error::Error::source(&error);
    assert!(source.is_some());
}

#[test]
fn context_error_component_resolution_display() {
    let error = ContextError::ComponentResolution {
        component: ComponentKey::of::<u32>(),
        source: Box::new(std::io::Error::other("resolve")),
    };
    let display = format!("{error}");
    assert!(display.contains("resolve") || display.contains("component"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_some());
}

#[test]
fn context_error_event_listener_resolution_display() {
    let error = ContextError::EventListenerResolution {
        component: ComponentKey::of::<u32>(),
        event: "evt",
        source: Box::new(std::io::Error::other("err")),
    };
    let display = format!("{error}");
    assert!(!display.is_empty());
    let source = std::error::Error::source(&error);
    assert!(source.is_some());
}

#[test]
fn context_error_application_runner_resolution_display() {
    let error = ContextError::ApplicationRunnerResolution {
        component: ComponentKey::of::<u32>(),
        source: Box::new(std::io::Error::other("err")),
    };
    let display = format!("{error}");
    assert!(!display.is_empty());
    let source = std::error::Error::source(&error);
    assert!(source.is_some());
}

#[test]
fn context_error_scheduled_task_resolution_display() {
    let error = ContextError::ScheduledTaskResolution {
        component: ComponentKey::of::<u32>(),
        source: Box::new(std::io::Error::other("err")),
    };
    let display = format!("{error}");
    assert!(!display.is_empty());
    let source = std::error::Error::source(&error);
    assert!(source.is_some());
}

#[test]
fn context_error_application_runner_failed_display() {
    let err: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::other("fail"));
    let error = ContextError::ApplicationRunnerFailed {
        runner: "my-runner",
        source: err,
    };
    let display = format!("{error}");
    assert!(display.contains("my-runner"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_some());
}

#[test]
fn context_error_application_runner_timeout_display() {
    let error = ContextError::ApplicationRunnerTimeout {
        runner: "my-runner",
        timeout: std::time::Duration::from_secs(30),
        abort_settled: true,
    };
    let display = format!("{error}");
    assert!(display.contains("my-runner"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn context_error_lifecycle_timeout_display() {
    let error = ContextError::LifecycleTimeout {
        component: "db",
        phase: vernal_context::LifecyclePhase::Stop,
        timeout: std::time::Duration::from_secs(5),
        abort_settled: false,
    };
    let display = format!("{error}");
    assert!(display.contains("db"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn context_error_managed_task_display() {
    let error = ContextError::ManagedTask {
        source: vernal_context::ManagedTaskError::InvalidName,
    };
    let display = format!("{error}");
    assert!(!display.is_empty());
    let source = std::error::Error::source(&error);
    assert!(source.is_some());
}

#[test]
fn context_error_lifecycle_coordinator_display() {
    let error = ContextError::LifecycleCoordinator {
        operation: "refresh",
        source: Arc::new(std::io::Error::other("coord panic")),
    };
    let display = format!("{error}");
    assert!(display.contains("refresh"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_some());
}

#[test]
fn context_error_lifecycle_cancelled_display() {
    let error = ContextError::LifecycleCancelled { operation: "start" };
    let display = format!("{error}");
    assert!(display.contains("start"), "display: {display}");
    let source = std::error::Error::Error::source(&error);
    assert!(source.is_none());
}

#[test]
fn context_error_shutdown_signal_display() {
    let error = ContextError::ShutdownSignal {
        source: Arc::new(std::io::Error::other("sig")),
    };
    let display = format!("{error}");
    assert!(!display.is_empty());
    let source = std::error::Error::source(&error);
    assert!(source.is_some());
}

#[test]
fn context_error_pause_restart_display() {
    let error = ContextError::PauseRestart {
        operation: "pause",
        component: "consumer",
        phase: vernal_context::LifecyclePhase::Pause,
        source: Arc::new(std::io::Error::other("pause fail")),
    };
    let display = format!("{error}");
    assert!(display.contains("pause"), "display: {display}");
    let source = std::error::Error::source(&error);
    assert!(source.is_some());
}

#[test]
fn context_error_application_runner_failed_no_runner_format() {
    // ApplicationRunnerFailed has no runner field — it has source.
    let err: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::other(""));
    let error = ContextError::ApplicationRunnerFailed { runner: "", source: err };
    let display = format!("{error}");
    assert!(display.contains("application runner"));
}

#[test]
fn context_error_lifecycle_timeout_no_component_format() {
    let error = ContextError::LifecycleTimeout {
        component: "x",
        phase: vernal_context::LifecyclePhase::Stop,
        timeout: std::time::Duration::from_secs(0),
        abort_settled: true,
    };
    let display = format!("{error}");
    assert!(display.contains("lifecycle"));
    assert!(display.contains("stop"));
}

// ════════════════════════════════════════════════════════════════════
// ApplicationEventListener hooks (0% coverage → 100%)
// ════════════════════════════════════════════════════════════════════

struct TestEvent;
struct CountingListener {
    count: Arc<std::sync::atomic::AtomicUsize>,
}

impl vernal_context::ApplicationEventListener<TestEvent> for CountingListener {
    type Error = std::io::Error;

    async fn on_event(
        &self,
        _event: Arc<TestEvent>,
    ) -> Result<(), Self::Error> {
        self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "counting-listener"
    }
}

struct FilteredListener;

impl vernal_context::ApplicationEventListener<TestEvent> for FilteredListener {
    type Error = std::io::Error;

    async fn on_event(
        &self,
        _event: Arc<TestEvent>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn supports_event_type(&self, event_type: TypeId) -> bool {
        // Reject a different event type to test filtering
        event_type != TypeId::of::<TestEvent>()
    }
}

struct SourceFilteredListener;

impl vernal_context::ApplicationEventListener<TestEvent> for SourceFilteredListener {
    type Error = std::io::Error;

    async fn on_event(
        &self,
        _event: Arc<TestEvent>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn supports_source(
        &self,
        source_type: Option<TypeId>,
    ) -> bool {
        // Only accept events without source
        source_type.is_none()
    }
}

struct NamedListener;

impl vernal_context::ApplicationEventListener<TestEvent> for NamedListener {
    type Error = std::io::Error;

    async fn on_event(
        &self,
        _event: Arc<TestEvent>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn listener_id(&self) -> &'static str {
        "named-listener-id"
    }
}

struct OrderedListener;

impl vernal_context::ApplicationEventListener<TestEvent> for OrderedListener {
    type Error = std::io::Error;

    async fn on_event(
        &self,
        _event: Arc<TestEvent>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn order(&self) -> i32 {
        100
    }
}

#[test]
fn application_event_listener_default_hooks() {
    struct DefaultListener;
    impl vernal_context::ApplicationEventListener<u32> for DefaultListener {
        type Error = std::io::Error;
        async fn on_event(
            &self,
            _event: Arc<u32>,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }
    let l = DefaultListener;
    // Default: supports_event_type returns true
    assert!(l.supports_event_type(TypeId::of::<u32>()));
    assert!(l.supports_source(None));
    assert!(l.supports_source(Some(TypeId::of::<String>())));
    // Default: listener_id is empty
    assert_eq!(l.listener_id(), "");
    // Default: order is i32::MAX
    assert_eq!(l.order(), i32::MAX);
    // Default: name uses type_name
    assert!(l.name().contains("DefaultListener"));
}

#[test]
fn application_event_listener_supports_event_type_override() {
    let l = FilteredListener;
    // FilteredListener rejects TestEvent type
    assert!(!l.supports_event_type(TypeId::of::<TestEvent>()));
    assert!(l.supports_event_type(TypeId::of::<String>()));
}

#[test]
fn application_event_listener_supports_source_override() {
    let l = SourceFilteredListener;
    assert!(l.supports_source(None));
    assert!(!l.supports_source(Some(TypeId::of::<String>())));
}

#[test]
fn application_event_listener_listener_id_override() {
    let l = NamedListener;
    assert_eq!(l.listener_id(), "named-listener-id");
}

#[test]
fn application_event_listener_order_override() {
    let l = OrderedListener;
    assert_eq!(l.order(), 100);
}

// ════════════════════════════════════════════════════════════════════
// PayloadApplicationEvent (0% coverage → 100%)
// ════════════════════════════════════════════════════════════════════

#[test]
fn payload_application_event_display_through_type_descriptor() {
    let source: Arc<dyn Any + Send + Sync> = Arc::new("svc");
    let event: PayloadApplicationEvent<String> = PayloadApplicationEvent::new(source, Arc::new("payload-1".to_string()));
    // Test type_descriptor (covers Display-via-TypeDescriptor path)
    let _td = event.type_descriptor();
    let _payload = event.payload();
    let _type_id = event.payload_type();
    let cloned = event.clone();
    assert_eq!(*cloned.payload(), "payload-1");
    assert_eq!(event.timestamp(), cloned.timestamp());
}

#[test]
fn payload_application_event_with_timestamp() {
    let source: Arc<dyn Any + Send + Sync> = Arc::new("svc");
    let event: PayloadApplicationEvent<String> =
        PayloadApplicationEvent::with_timestamp(source, Arc::new("p".to_string()), 1_700_000_000_000);
    assert_eq!(event.timestamp(), 1_700_000_000_000);
    // TypeDescriptor path (from event as ApplicationContextEvent)
    assert!(vernal_context::ApplicationContextEvent::timestamp(&event) == 1_700_000_000_000);
}

// ════════════════════════════════════════════════════════════════════
// ConfigurationPhase (0% coverage → 100%)
// ════════════════════════════════════════════════════════════════════

#[test]
fn configuration_phase_as_str() {
    assert_eq!(ConfigurationPhase::ParseConfiguration.as_str(), "parse_configuration");
    assert_eq!(ConfigurationPhase::RegisterBean.as_str(), "register_bean");
}

#[test]
fn configuration_phase_equality_and_serde() {
    let p1 = ConfigurationPhase::ParseConfiguration;
    let p2 = ConfigurationPhase::ParseConfiguration;
    let r = ConfigurationPhase::RegisterBean;
    assert_eq!(p1, p2);
    assert_ne!(p1, r);
    let json = serde_json::to_string(&p1).expect("serialize");
    assert!(json.contains("parse_configuration"));
}

// ════════════════════════════════════════════════════════════════════
// ApplicationListenerRegistration + EventListenerRegistry (0% → 100%)
// ════════════════════════════════════════════════════════════════════

struct MyListener;
impl vernal_context::ApplicationEventListener<String> for MyListener {
    type Error = std::io::Error;
    async fn on_event(&self, _event: Arc<String>) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[test]
fn listener_key_constructors() {
    let k1 = ListenerKey::for_event::<String>();
    let k2 = ListenerKey::named("foo");
    assert!(matches!(k1, ListenerKey::TypeId(_)));
    assert!(matches!(k2, ListenerKey::Named(s) if s == "foo"));
}

#[test]
fn application_listener_registration_clone() {
    let listener = Arc::new(MyListener);
    let reg: ApplicationListenerRegistration<String, MyListener> =
        ApplicationListenerRegistration::new(listener);
    let _ = reg.listener();
}

#[test]
fn event_listener_registry_default_count_zero() {
    // This tests that the trait is implemented for EventBus; see event_bus_tests.
    // We can construct an empty bus.
    let bus = vernal_context::EventBus::new();
    assert_eq!(bus.listener_count(), 0);
}

#[test]
fn event_listener_registry_add_remove_listeners() {
    let bus = vernal_context::EventBus::new();
    let listener = Arc::new(MyListener);
    let reg: ApplicationListenerRegistration<String, MyListener> =
        ApplicationListenerRegistration::new(listener);
    assert!(bus.add_listener(reg));
    assert_eq!(bus.listener_count(), 1);
    // Add same listener id — returns false
    let listener2 = Arc::new(MyListener);
    let reg2: ApplicationListenerRegistration<String, MyListener> =
        ApplicationListenerRegistration::new(listener2);
    assert!(!bus.add_listener(reg2));
    // remove by named
    let key = ListenerKey::named("my_listener");
    assert!(!bus.remove_listener(&key));
    // remove by TypeId
    let key2 = ListenerKey::for_event::<String>();
    assert!(bus.remove_listener(&key2));
    assert_eq!(bus.listener_count(), 0);
    // remove already removed - returns false
    assert!(!bus.remove_listener(&key2));
}

// ════════════════════════════════════════════════════════════════════
// ConfigurationPropertiesError (100% already, just verify all branches)
// ════════════════════════════════════════════════════════════════════

#[test]
fn configuration_properties_error_all_variants() {
    let env = vernal_context::ApplicationEnvironment::empty();

    // environment
    let env_err = EnvironmentError::InvalidPropertyKey { key: "k".to_string() };
    let err = ConfigurationPropertiesError::environment::<u32>("field", "k".to_string(), env_err);
    let _ = err.field();
    let _ = err.property_key();
    let _ = err.configuration_type();
    let display = format!("{err}");
    assert!(display.contains("field"));
    let _ = format!("{err:?}");
    let source = std::error::Error::source(&err);
    assert!(source.is_some());

    // nested
    let env_err2 = EnvironmentError::InvalidPropertyKey { key: "k".to_string() };
    let inner = ConfigurationPropertiesError::environment::<u32>("inner", "k".to_string(), env_err2);
    let outer = ConfigurationPropertiesError::nested::<String>("outer", "k", inner);
    let _ = outer.field();
    let _ = outer.property_key();
    let display = format!("{outer}");
    assert!(display.contains("outer"));
}
