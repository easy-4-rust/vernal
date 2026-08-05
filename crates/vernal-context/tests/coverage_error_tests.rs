//! 覆盖率提升测试 - 覆盖 ApplicationBuildError 和其他错误类型

use std::error::Error;

use vernal_beans::{ComponentKey, DefinitionError, GraphError, ResolveError};
use vernal_context::{ApplicationBuildError, ConditionError, ContextError, ContextState};

// ════════════════════════════════════════════════════════════════════
// 测试: ApplicationBuildError Display variants
// ════════════════════════════════════════════════════════════════════

/// 验证 ApplicationBuildError::Definition Display
#[test]
fn test_build_error_definition_display() {
    let err = ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier {
            value: "test".to_string(),
        },
    };
    let display = format!("{}", err);
    assert!(!display.is_empty());
}

/// 验证 ApplicationBuildError::Graph Display
#[test]
fn test_build_error_graph_display() {
    let err = ApplicationBuildError::Graph {
        source: GraphError::MissingDependency {
            path: vec!["test".to_string()],
        },
    };
    let display = format!("{}", err);
    assert!(!display.is_empty());
}

/// 验证 ApplicationBuildError::Context Display
#[test]
fn test_build_error_context_display() {
    let err = ApplicationBuildError::Context {
        source: ContextError::InvalidState {
            operation: "refresh",
            state: ContextState::Created,
        },
    };
    let display = format!("{}", err);
    assert!(!display.is_empty());
}

/// 验证 ApplicationBuildError::Condition Display
#[test]
fn test_build_error_condition_display() {
    let err = ApplicationBuildError::Condition {
        source: ConditionError::DuplicateModule { name: "test" },
    };
    let display = format!("{}", err);
    assert!(!display.is_empty());
}

/// 验证 ApplicationBuildError::AdvisorResolution Display
#[test]
fn test_build_error_advisor_resolution_display() {
    let err = ApplicationBuildError::AdvisorResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(ResolveError::NotFound {
            component: "test".to_string(),
            path: vec!["test".to_string()],
        }),
    };
    let display = format!("{}", err);
    assert!(!display.is_empty());
}

/// 验证 ApplicationBuildError::LocalAdvisorResolution Display
#[test]
fn test_build_error_local_advisor_resolution_display() {
    let err = ApplicationBuildError::LocalAdvisorResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(ResolveError::NotFound {
            component: "test".to_string(),
            path: vec!["test".to_string()],
        }),
    };
    let display = format!("{}", err);
    assert!(!display.is_empty());
}

/// 验证 ApplicationBuildError::AdvisorScope Display
#[test]
fn test_build_error_advisor_scope_display() {
    let err = ApplicationBuildError::AdvisorScope {
        component: ComponentKey::of::<String>(),
        scope: "singleton",
    };
    let display = format!("{}", err);
    assert!(!display.is_empty());
}

/// 验证 ApplicationBuildError::source() 方法
#[test]
fn test_build_error_source() {
    let err = ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier {
            value: "test".to_string(),
        },
    };
    let source = err.source();
    assert!(source.is_some());
}

/// 验证 ApplicationBuildError::TokioRuntimeUnavailable Display
#[test]
fn test_build_error_tokio_runtime_unavailable_display() {
    let source = tokio::runtime::Handle::try_current().err().unwrap();
    let err = ApplicationBuildError::TokioRuntimeUnavailable { source };
    let display = format!("{}", err);
    assert!(!display.is_empty());
}

/// 验证 ApplicationBuildError::TokioRuntimeUnavailable source
#[test]
fn test_build_error_tokio_runtime_unavailable_source() {
    let source = tokio::runtime::Handle::try_current().err().unwrap();
    let err = ApplicationBuildError::TokioRuntimeUnavailable { source };
    let source = err.source();
    assert!(source.is_some());
}

/// 验证 ApplicationBuildError::Definition source
#[test]
fn test_build_error_definition_source() {
    let err = ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier {
            value: "test".to_string(),
        },
    };
    let source = err.source();
    assert!(source.is_some());
}

/// 验证 ApplicationBuildError::Graph source
#[test]
fn test_build_error_graph_source() {
    let err = ApplicationBuildError::Graph {
        source: GraphError::MissingDependency {
            path: vec!["test".to_string()],
        },
    };
    let source = err.source();
    assert!(source.is_some());
}

/// 验证 ApplicationBuildError::Context source
#[test]
fn test_build_error_context_source() {
    let err = ApplicationBuildError::Context {
        source: ContextError::InvalidState {
            operation: "refresh",
            state: ContextState::Created,
        },
    };
    let source = err.source();
    assert!(source.is_some());
}

/// 验证 ApplicationBuildError::Condition source
#[test]
fn test_build_error_condition_source() {
    let err = ApplicationBuildError::Condition {
        source: ConditionError::DuplicateModule { name: "test" },
    };
    let source = err.source();
    assert!(source.is_some());
}

/// 验证 ApplicationBuildError::AdvisorResolution source
#[test]
fn test_build_error_advisor_resolution_source() {
    let err = ApplicationBuildError::AdvisorResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(ResolveError::NotFound {
            component: "test".to_string(),
            path: vec!["test".to_string()],
        }),
    };
    let source = err.source();
    assert!(source.is_some());
}

/// 验证 ApplicationBuildError::LocalAdvisorResolution source
#[test]
fn test_build_error_local_advisor_resolution_source() {
    let err = ApplicationBuildError::LocalAdvisorResolution {
        component: ComponentKey::of::<String>(),
        source: Box::new(ResolveError::NotFound {
            component: "test".to_string(),
            path: vec!["test".to_string()],
        }),
    };
    let source = err.source();
    assert!(source.is_some());
}

/// 验证 ApplicationBuildError Debug
#[test]
fn test_build_error_debug() {
    let err = ApplicationBuildError::Definition {
        source: DefinitionError::InvalidQualifier {
            value: "test".to_string(),
        },
    };
    let debug = format!("{:?}", err);
    assert!(!debug.is_empty());
}
