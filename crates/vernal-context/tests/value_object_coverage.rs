//! 覆盖率补足：简单值对象（diagnostic_*, task_options, task_schedule_error,
//! subsystem_status, async_task, configuration_properties, lib::project_status）。
//!
//! 这些对象本身没有复杂业务逻辑，但它们的 `Display` / `as_str` / `new` /
//! `Default` / builder 方法在差分测试中从未被触发，导致覆盖率拉低。本测试文件
//! 逐一覆盖这些 public API，让覆盖率从 73.78% 提升。

use std::time::Duration;

use vernal_context::{
    AsyncTask, ConfigurationProperties, ConfigurationPropertiesError, DiagnosticOutcome,
    DiagnosticPhase, DiagnosticState, SubsystemStatus, TaskOptions, TaskPriority,
    TaskScheduleError,
};
use vernal_core::BoxError;

// ── DiagnosticOutcome ───────────────────────────────────────────────────

#[test]
fn diagnostic_outcome_as_str_round_trip() {
    assert_eq!(DiagnosticOutcome::Succeeded.as_str(), "succeeded");
    assert_eq!(DiagnosticOutcome::Failed.as_str(), "failed");
}

#[test]
fn diagnostic_outcome_serde_round_trip() {
    let json = serde_json::to_string(&DiagnosticOutcome::Succeeded).expect("serialize");
    assert_eq!(json, "\"succeeded\"");
    let json = serde_json::to_string(&DiagnosticOutcome::Failed).expect("serialize");
    assert_eq!(json, "\"failed\"");
}

// ── DiagnosticPhase ─────────────────────────────────────────────────────

#[test]
fn diagnostic_phase_as_str_covers_all_variants() {
    assert_eq!(DiagnosticPhase::ContainerWarmUp.as_str(), "container_warm_up");
    assert_eq!(DiagnosticPhase::ComponentResolution.as_str(), "component_resolution");
    assert_eq!(DiagnosticPhase::Initialize.as_str(), "initialize");
    assert_eq!(DiagnosticPhase::Start.as_str(), "start");
    assert_eq!(DiagnosticPhase::ApplicationRunner.as_str(), "application_runner");
    assert_eq!(
        DiagnosticPhase::ScheduledTaskActivation.as_str(),
        "scheduled_task_activation"
    );
    assert_eq!(DiagnosticPhase::Stop.as_str(), "stop");
    assert_eq!(DiagnosticPhase::Pause.as_str(), "pause");
}

#[test]
fn diagnostic_phase_serde_round_trip() {
    let json = serde_json::to_string(&DiagnosticPhase::Pause).expect("serialize");
    assert_eq!(json, "\"pause\"");
}

// ── DiagnosticState ─────────────────────────────────────────────────────

#[test]
fn diagnostic_state_as_str_covers_all_variants() {
    assert_eq!(DiagnosticState::Unknown.as_str(), "unknown");
    assert_eq!(DiagnosticState::Available.as_str(), "available");
    assert_eq!(DiagnosticState::Degraded.as_str(), "degraded");
    assert_eq!(DiagnosticState::Unavailable.as_str(), "unavailable");
}

#[test]
fn diagnostic_state_default_is_unknown() {
    let state = DiagnosticState::default();
    assert_eq!(state, DiagnosticState::Unknown);
}

#[test]
fn diagnostic_state_serde_round_trip() {
    let json = serde_json::to_string(&DiagnosticState::Degraded).expect("serialize");
    assert_eq!(json, "\"degraded\"");
}

// ── SubsystemStatus ─────────────────────────────────────────────────────

#[test]
fn subsystem_status_round_trip() {
    let status = SubsystemStatus::new("axum", DiagnosticState::Available);
    assert_eq!(status.name(), "axum");
    assert_eq!(status.state(), DiagnosticState::Available);
}

#[test]
fn subsystem_status_serde_round_trip() {
    let status = SubsystemStatus::new("redis", DiagnosticState::Degraded);
    let json = serde_json::to_string(&status).expect("serialize");
    assert!(json.contains("\"name\":\"redis\""));
    assert!(json.contains("\"state\":\"degraded\""));
}

// ── TaskOptions / TaskPriority ──────────────────────────────────────────

#[test]
fn task_options_default_has_documented_values() {
    let opts = TaskOptions::default();
    assert!(opts.timeout.is_none(), "default timeout must be None");
    assert_eq!(opts.max_retries, 0, "default max_retries must be 0");
    assert_eq!(opts.priority, TaskPriority::Normal);
}

#[test]
fn task_options_new_matches_default() {
    let new_opts = TaskOptions::new();
    let default_opts = TaskOptions::default();
    assert!(new_opts.timeout.is_none());
    assert_eq!(new_opts.max_retries, default_opts.max_retries);
    assert_eq!(new_opts.priority, default_opts.priority);
}

#[test]
fn task_options_builder_chain() {
    let opts = TaskOptions::new()
        .with_timeout(Duration::from_secs(5))
        .with_max_retries(3)
        .with_priority(TaskPriority::High);
    assert_eq!(opts.timeout, Some(Duration::from_secs(5)));
    assert_eq!(opts.max_retries, 3);
    assert_eq!(opts.priority, TaskPriority::High);
}

#[test]
fn task_priority_ordering() {
    use TaskPriority::*;
    assert!(Low < Normal);
    assert!(Normal < High);
    assert!(High < Critical);
}

// ── TaskScheduleError ───────────────────────────────────────────────────

#[test]
fn task_schedule_error_zero_interval_display() {
    let error = TaskScheduleError::ZeroInterval;
    assert_eq!(
        format!("{error}"),
        "scheduled task interval must be non-zero"
    );
}

#[test]
fn task_schedule_error_is_std_error() {
    let error = TaskScheduleError::ZeroInterval;
    let _: &dyn std::error::Error = &error;
}

// ── AsyncTask trait default name ────────────────────────────────────────

struct SampleAsyncTask;

impl AsyncTask for SampleAsyncTask {
    fn run(
        &self,
        _token: tokio_util::sync::CancellationToken,
    ) -> vernal_context::LifecycleFuture<'_> {
        Box::pin(async { Ok(()) })
    }
}

#[test]
fn async_task_default_name_uses_type_name() {
    let task = SampleAsyncTask;
    assert!(task.name().contains("SampleAsyncTask"));
}

// ── project_status (lib.rs) ─────────────────────────────────────────────

#[test]
fn project_status_returns_static_str() {
    let status = vernal_context::project_status();
    // 实际值由 vernal_core::PROJECT_STATUS 决定，至少应该是非空静态字符串。
    assert!(!status.is_empty());
}

// ── ConfigurationPropertiesError ────────────────────────────────────────

#[test]
fn configuration_properties_error_environment_factory_and_display() {
    use vernal_context::EnvironmentError;

    let error = ConfigurationPropertiesError::environment::<u32>(
        "timeout",
        "app.timeout".to_string(),
        EnvironmentError::MissingProperty {
            key: "app.timeout".to_string(),
        },
    );
    assert_eq!(error.field(), "timeout");
    assert_eq!(error.property_key(), "app.timeout");
    // configuration_type 是 type_name::<u32>()，至少包含 "u32"。
    assert!(error.configuration_type().contains("u32"));

    let display = format!("{error}");
    assert!(display.contains("app.timeout"));
    assert!(display.contains("timeout"));
    assert!(display.contains("cannot bind field"));

    // source 链可访问 EnvironmentError。
    let source = std::error::Error::source(&error);
    assert!(source.is_some());
}

#[test]
fn configuration_properties_error_nested_factory() {
    use vernal_context::EnvironmentError;

    let inner = ConfigurationPropertiesError::environment::<u32>(
        "inner",
        "a.b".to_string(),
        EnvironmentError::MissingProperty {
            key: "a.b".to_string(),
        },
    );
    let outer =
        ConfigurationPropertiesError::nested::<String>("outer", "a".to_string(), inner);
    assert_eq!(outer.field(), "outer");
    assert_eq!(outer.property_key(), "a");
    // 嵌套错误仍可 Display。
    let display = format!("{outer}");
    assert!(display.contains("a"));
}

#[test]
fn configuration_properties_error_debug_redacts_source() {
    use vernal_context::EnvironmentError;

    let error = ConfigurationPropertiesError::environment::<u32>(
        "port",
        "app.port".to_string(),
        EnvironmentError::MissingProperty {
            key: "app.port".to_string(),
        },
    );
    let debug = format!("{error:?}");
    assert!(debug.contains("ConfigurationPropertiesError"));
    assert!(debug.contains("port"));
    // source 在 Debug 中被脱敏。
    assert!(debug.contains("<redacted>"));
}

// ── ConfigurationProperties component_definition smoke test ─────────────

#[test]
fn configuration_properties_component_definition_does_not_panic() {
    use vernal_context::ApplicationEnvironment;

    struct EmptyConfig;

    impl ConfigurationProperties for EmptyConfig {
        const PREFIX: &'static str = "empty";

        fn bind_with_prefix(
            _env: &ApplicationEnvironment,
            _prefix: &str,
        ) -> Result<Self, ConfigurationPropertiesError> {
            Ok(EmptyConfig)
        }
    }

    // component_definition 是关联函数；调用应不 panic。
    let _definition = EmptyConfig::component_definition();
}

// ── additional coverage for BoxError helper trait ───────────────────────

#[test]
fn box_error_can_be_constructed_from_io_error() {
    let _: BoxError = Box::new(std::io::Error::other("test"));
}