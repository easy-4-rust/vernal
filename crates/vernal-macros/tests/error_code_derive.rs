//! `#[derive(ErrorCode)]` 宏集成测试。

use vernal_core::error::{ErrorCode, VernalError};
use vernal_macros::ErrorCode;

/// 测试基本的 ErrorCode 派生：域、码、消息正确映射。
#[derive(ErrorCode, Debug, Clone, Copy, PartialEq, Eq)]
#[error("ioc")]
pub enum IoCErrorCode {
    /// 组件未找到
    #[error(-1, "组件未找到")]
    NotFound,

    /// 依赖歧义
    #[error(-2, "依赖歧义：找到多个候选")]
    Ambiguous,

    /// 循环依赖
    #[error(-3, "检测到循环依赖")]
    CircularDependency,
}

#[test]
fn domain_is_correct() {
    assert_eq!(IoCErrorCode::NotFound.domain(), "ioc");
    assert_eq!(IoCErrorCode::Ambiguous.domain(), "ioc");
    assert_eq!(IoCErrorCode::CircularDependency.domain(), "ioc");
}

#[test]
fn code_is_correct() {
    assert_eq!(IoCErrorCode::NotFound.code(), -1);
    assert_eq!(IoCErrorCode::Ambiguous.code(), -2);
    assert_eq!(IoCErrorCode::CircularDependency.code(), -3);
}

#[test]
fn message_is_correct() {
    assert_eq!(IoCErrorCode::NotFound.message(), "组件未找到");
    assert_eq!(IoCErrorCode::Ambiguous.message(), "依赖歧义：找到多个候选");
    assert_eq!(IoCErrorCode::CircularDependency.message(), "检测到循环依赖");
}

#[test]
fn display_format() {
    assert_eq!(format!("{}", IoCErrorCode::NotFound), "[ioc:-1] 组件未找到");
}

#[test]
fn is_same_kind_works() {
    // 同类：相同 domain + code
    assert!(IoCErrorCode::NotFound.is_same_kind(&IoCErrorCode::NotFound));

    // 不同类：不同 code
    assert!(!IoCErrorCode::NotFound.is_same_kind(&IoCErrorCode::Ambiguous));
}

#[test]
fn into_vernal_error_preserves_identity() {
    let err: VernalError = IoCErrorCode::Ambiguous.into_vernal_error();
    assert_eq!(err.domain(), Some("ioc"));
    assert_eq!(err.code(), Some(-2));
    assert_eq!(err.message(), "依赖歧义：找到多个候选");
}

#[test]
fn from_trait_conversion() {
    let err: VernalError = IoCErrorCode::NotFound.into();
    assert_eq!(err.domain(), Some("ioc"));
    assert_eq!(err.code(), Some(-1));
}

/// 测试带数据的变体（Tuple / Struct）。
#[derive(ErrorCode, Debug)]
#[error("context")]
pub enum ContextErrorCode {
    /// 带元组数据的变体
    #[error(-1, "配置缺失")]
    MissingConfig(String),

    /// 带命名字段的变体
    #[error(-2, "生命周期失败")]
    LifecycleFailed { phase: &'static str },

    /// 单元变体
    #[error(-3, "事件总线已关闭")]
    EventBusClosed,
}

#[test]
fn tuple_variant_matches() {
    let err = ContextErrorCode::MissingConfig("database.url".to_string());
    assert_eq!(err.domain(), "context");
    assert_eq!(err.code(), -1);
    assert_eq!(err.message(), "配置缺失");
}

#[test]
fn struct_variant_matches() {
    let err = ContextErrorCode::LifecycleFailed { phase: "start" };
    assert_eq!(err.domain(), "context");
    assert_eq!(err.code(), -2);
}

#[test]
fn unit_variant_matches() {
    let err = ContextErrorCode::EventBusClosed;
    assert_eq!(err.domain(), "context");
    assert_eq!(err.code(), -3);
    assert_eq!(err.message(), "事件总线已关闭");
}

/// 测试 Error trait 实现（source 为 None）。
#[test]
fn error_trait_source_is_none() {
    let err = IoCErrorCode::NotFound;
    assert!(std::error::Error::source(&err).is_none());
}
