//! ApplicationLaunchError 测试 - 覆盖所有错误变体

use std::error::Error;

use vernal_context::{
    ApplicationBuildError, ApplicationLaunchError, ConditionError, ContextError, ContextState,
};
use vernal_beans::DefinitionError;

// ════════════════════════════════════════════════════════════════════
// 测试: ApplicationLaunchError::Build
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_launch_error_build_display() {
    let err = ApplicationLaunchError::build(ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier {
            value: "test".to_string(),
        },
    });
    let display = format!("{err}");
    assert!(display.contains("build failed"));
}

#[test]
fn test_launch_error_build_source() {
    let err = ApplicationLaunchError::build(ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier {
            value: "test".to_string(),
        },
    });
    assert!(err.source().is_some());
}

#[test]
fn test_launch_error_build_operation() {
    let err = ApplicationLaunchError::build(ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier {
            value: "test".to_string(),
        },
    });
    assert_eq!(err.operation(), "build");
}

#[test]
fn test_launch_error_build_startup_report_none() {
    let err = ApplicationLaunchError::build(ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier {
            value: "test".to_string(),
        },
    });
    assert!(err.startup_report().is_none());
}

#[test]
fn test_launch_error_build_cleanup_error_none() {
    let err = ApplicationLaunchError::build(ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier {
            value: "test".to_string(),
        },
    });
    assert!(err.cleanup_error().is_none());
}

#[test]
fn test_launch_error_build_debug() {
    let err = ApplicationLaunchError::build(ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier {
            value: "test".to_string(),
        },
    });
    let debug = format!("{err:?}");
    assert!(debug.contains("Build"));
}

#[test]
fn test_launch_error_build_is_std_error() {
    let err = ApplicationLaunchError::build(ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier {
            value: "test".to_string(),
        },
    });
    let _: &dyn Error = &err;
}

// ════════════════════════════════════════════════════════════════════
// 测试: ApplicationLaunchError 其他变体 (通过 build 间接测试)
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_launch_error_build_with_condition_error() {
    let err = ApplicationLaunchError::build(ApplicationBuildError::Condition {
        source: ConditionError::DuplicateModule {
            name: "test",
        },
    });
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn test_launch_error_build_with_context_error() {
    let err = ApplicationLaunchError::build(ApplicationBuildError::Context {
        source: ContextError::InvalidState {
            operation: "refresh",
            state: ContextState::Created,
        },
    });
    let display = format!("{err}");
    assert!(!display.is_empty());
}
