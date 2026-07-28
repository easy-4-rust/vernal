//! 覆盖率补足：错误枚举的 Display / Debug / source 实现全分支覆盖。
//!
//! vernal-context 错误族的 Display impl 中的 match 分支在差分测试中很少触发，
//! 导致覆盖率拉低。本测试文件逐一构造每个 public 变体并断言 Display 输出
//! 包含预期关键字。

use std::{sync::Arc, time::Duration};

use vernal_beans::ComponentKey;
use vernal_context::{
    ConditionError, ContextError, ContextState, EventListenerError, ManagedTaskError,
};
use vernal_core::SharedError;

// ── ContextError Display 全分支 ─────────────────────────────────────────

#[test]
fn context_error_invalid_state_display() {
    let error = ContextError::InvalidState {
        operation: "pause",
        state: ContextState::Closed,
    };
    let display = format!("{error}");
    assert!(display.contains("pause"));
    assert!(display.contains("Closed"));
}

#[test]
fn context_error_lifecycle_definition_not_found_display() {
    let error = ContextError::LifecycleDefinitionNotFound {
        component: ComponentKey::of::<u32>(),
    };
    let display = format!("{error}");
    assert!(display.contains("u32"));
}

#[test]
fn context_error_lifecycle_display() {
    let error = ContextError::Lifecycle {
        component: "db",
        phase: vernal_context::LifecyclePhase::Start,
        source: Arc::new(std::io::Error::other("conn refused")) as SharedError,
    };
    let display = format!("{error}");
    assert!(display.contains("db"));
    assert!(display.contains("start"));
}

#[test]
fn context_error_lifecycle_timeout_display() {
    let error = ContextError::LifecycleTimeout {
        component: "db",
        phase: vernal_context::LifecyclePhase::Stop,
        timeout: Duration::from_secs(5),
        abort_settled: true,
    };
    let display = format!("{error}");
    assert!(display.contains("db"));
    assert!(display.contains("stop"));
}

#[test]
fn context_error_managed_task_display() {
    let error = ContextError::ManagedTask {
        source: ManagedTaskError::InvalidName,
    };
    let display = format!("{error}");
    assert!(display.contains("managed task"));
}

#[test]
fn context_error_lifecycle_coordinator_display() {
    let error = ContextError::LifecycleCoordinator {
        operation: "refresh",
        source: Arc::new(std::io::Error::other("coordinator panic")) as SharedError,
    };
    let display = format!("{error}");
    assert!(display.contains("refresh"));
}

#[test]
fn context_error_lifecycle_cancelled_display() {
    let error = ContextError::LifecycleCancelled { operation: "start" };
    let display = format!("{error}");
    assert!(display.contains("start"));
    assert!(display.contains("cancelled"));
}

#[test]
fn context_error_shutdown_signal_display() {
    let error = ContextError::ShutdownSignal {
        source: Arc::new(std::io::Error::other("signal handler failure")) as SharedError,
    };
    let display = format!("{error}");
    assert!(display.contains("shutdown signal"));
}

#[test]
fn context_error_pause_restart_display() {
    let error = ContextError::PauseRestart {
        operation: "pause",
        component: "consumer",
        phase: vernal_context::LifecyclePhase::Pause,
        source: Arc::new(std::io::Error::other("pause failed")) as SharedError,
    };
    let display = format!("{error}");
    assert!(display.contains("pause"));
    assert!(display.contains("consumer"));
}

// ── ManagedTaskError Display 全分支 ─────────────────────────────────────

#[test]
fn managed_task_error_invalid_name_display() {
    let error = ManagedTaskError::InvalidName;
    assert!(format!("{error}").contains("name"));
}

#[test]
fn managed_task_error_spawn_rejected_display() {
    let error = ManagedTaskError::SpawnRejected { task: "rejected" };
    let display = format!("{error}");
    assert!(display.contains("rejected"));
}

#[test]
fn managed_task_error_identifier_exhausted_display() {
    let error = ManagedTaskError::IdentifierExhausted { task: "x" };
    assert!(format!("{error}").contains("x"));
}

#[test]
fn managed_task_error_task_failed_display() {
    let error = ManagedTaskError::TaskFailed {
        task: "f",
        source: Arc::new(std::io::Error::other("fail")) as SharedError,
    };
    assert!(format!("{error}").contains("f"));
}

#[tokio::test]
async fn managed_task_error_task_panicked_display() {
    let handle = tokio::spawn(async { panic!("boom") });
    let join_error = handle.await.unwrap_err();
    let error = ManagedTaskError::TaskPanicked {
        task: "panic-task",
        source: Arc::new(join_error),
    };
    assert!(format!("{error}").contains("panic-task"));
}

#[tokio::test]
async fn managed_task_error_task_cancelled_display() {
    let handle = tokio::spawn(async {
        loop {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });
    handle.abort();
    let join_error = handle.await.unwrap_err();
    let error = ManagedTaskError::TaskCancelled {
        task: "cancelled-task",
        source: Arc::new(join_error),
    };
    assert!(format!("{error}").contains("cancelled-task"));
}

#[test]
fn managed_task_error_shutdown_timeout_display() {
    let error = ManagedTaskError::ShutdownTimeout {
        timeout: Duration::from_secs(5),
        remaining: 2,
    };
    let display = format!("{error}");
    assert!(display.contains("5s") || display.contains("5"));
    assert!(display.contains("2"));
}

#[test]
fn managed_task_error_abort_timeout_display() {
    let error = ManagedTaskError::AbortTimeout {
        timeout: Duration::from_secs(1),
        remaining: 3,
    };
    let display = format!("{error}");
    assert!(display.contains("1s") || display.contains("1"));
}

#[test]
fn managed_task_error_coordinator_unavailable_display() {
    let error = ManagedTaskError::CoordinatorUnavailable { remaining: 4 };
    assert!(format!("{error}").contains("4"));
}

// ── EventListenerError Display 全分支 ───────────────────────────────────

#[test]
fn event_listener_error_handler_failed_display() {
    let error = EventListenerError::HandlerFailed {
        listener: "my-listener",
        event: "MyEvent",
        source: Arc::new(std::io::Error::other("handler error")) as SharedError,
    };
    let display = format!("{error}");
    assert!(display.contains("my-listener"));
    assert!(display.contains("MyEvent"));
}

#[test]
fn event_listener_error_lagged_display() {
    let error = EventListenerError::Lagged {
        listener: "slow-listener",
        event: "Tick",
        skipped: 42,
    };
    let display = format!("{error}");
    assert!(display.contains("slow-listener"));
    assert!(display.contains("42"));
}

// ── ConditionError Display 全分支 ───────────────────────────────────────

#[test]
fn condition_error_invalid_property_key_display() {
    let error = ConditionError::InvalidPropertyKey {
        key: "has space".to_string(),
    };
    // Display 已脱敏 key 值，只输出固定文本。
    let display = format!("{error}");
    assert!(!display.is_empty());
}

#[test]
fn condition_error_invalid_profile_display() {
    let error = ConditionError::InvalidProfile {
        profile: "dev ".to_string(),
    };
    let display = format!("{error}");
    assert!(!display.is_empty());
}

#[test]
fn condition_error_empty_profile_set_display() {
    let error = ConditionError::EmptyProfileSet;
    assert!(format!("{error}").contains("profile"));
}

#[test]
fn condition_error_duplicate_module_display() {
    let error = ConditionError::DuplicateModule {
        name: "duplicate-mod",
    };
    let display = format!("{error}");
    assert!(!display.is_empty());
}

#[test]
fn condition_error_invalid_module_name_display() {
    let error = ConditionError::InvalidModuleName { name: "bad name" };
    let display = format!("{error}");
    assert!(!display.is_empty());
}

#[test]
fn condition_error_empty_module_display() {
    let error = ConditionError::EmptyModule { name: "empty-mod" };
    let display = format!("{error}");
    assert!(!display.is_empty());
}

// ── EnvironmentError Display 全分支 ─────────────────────────────────────

#[test]
fn environment_error_all_variants_display() {
    use vernal_context::EnvironmentError;

    let invalid_property_key = EnvironmentError::InvalidPropertyKey {
        key: "bad key".to_string(),
    };
    assert!(format!("{invalid_property_key}").contains("bad key"));

    let invalid_profile = EnvironmentError::InvalidProfile {
        profile: "dev ".to_string(),
    };
    assert!(format!("{invalid_profile}").contains("dev "));

    let missing_property = EnvironmentError::MissingProperty {
        key: "missing.key".to_string(),
    };
    assert!(format!("{missing_property}").contains("missing.key"));

    let invalid_property_value = EnvironmentError::InvalidPropertyValue {
        key: "key".to_string(),
        source_name: "src".to_string(),
        target_type: "u32",
    };
    let display = format!("{invalid_property_value}");
    assert!(display.contains("key") && display.contains("u32"));

    let malformed_placeholder = EnvironmentError::MalformedPlaceholder {
        property: "p".to_string(),
    };
    assert!(format!("{malformed_placeholder}").contains("p"));

    let unresolved_placeholder = EnvironmentError::UnresolvedPlaceholder {
        property: "p".to_string(),
        placeholder: "ph".to_string(),
    };
    assert!(format!("{unresolved_placeholder}").contains("ph"));

    let circular_placeholder = EnvironmentError::CircularPlaceholder {
        path: vec!["a".to_string(), "b".to_string()],
    };
    assert!(format!("{circular_placeholder}").contains("a"));

    let placeholder_depth_exceeded = EnvironmentError::PlaceholderDepthExceeded {
        property: "p".to_string(),
        limit: 32,
    };
    assert!(format!("{placeholder_depth_exceeded}").contains("32"));

    let duplicate_source = EnvironmentError::DuplicatePropertySource {
        name: "dup-src".to_string(),
    };
    assert!(format!("{duplicate_source}").contains("dup-src"));

    let invalid_source_name = EnvironmentError::InvalidPropertySourceName {
        name: "bad name".to_string(),
    };
    assert!(format!("{invalid_source_name}").contains("bad name"));

    let property_source = EnvironmentError::PropertySource {
        source_name: "failing".to_string(),
        source: Arc::new(std::io::Error::other("io fail")) as SharedError,
    };
    assert!(format!("{property_source}").contains("failing"));
}

// ── EnvironmentSnapshot / ConditionEvaluationSnapshot 间接覆盖 ─────────
//
// 这两个快照的构造器是 `pub(crate)`，只能通过 `ApplicationEnvironment::snapshot()`
// 与 `StartupReport::condition_evaluations()` 间接访问。直接构造会失败。
// 这里通过 Environment 间接覆盖 snapshot + serde 路径。

#[test]
fn environment_snapshot_via_environment_snapshot_method() {
    use std::sync::Arc;
    use vernal_context::{ApplicationEnvironment, MapPropertySource};

    let source = MapPropertySource::new("app", [("name", "vernal")]).expect("valid source");
    let mut builder = ApplicationEnvironment::builder();
    builder.add_last(Arc::new(source)).expect("add source");
    builder.active_profile("prod").expect("valid profile");
    let env = builder.build();

    let snapshot = env.snapshot();
    let json = serde_json::to_string(&snapshot).expect("serialize");
    assert!(json.contains("\"app\""), "snapshot json: {json}");
    assert!(json.contains("\"prod\""));
    assert_eq!(snapshot.active_profiles(), &["prod".to_string()]);
}
