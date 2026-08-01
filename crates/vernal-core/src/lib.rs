#![allow(unused)] // deny(unsafe_code) 移除：Rust 2024 edition 中 set_var 需要 unsafe 块
#![doc = "Vernal 各独立内核共享的稳定基础合同。"]

pub mod alias_registry;
pub mod app_lifecycle_phase;
pub mod async_runtime;
pub mod attribute_accessor;
pub mod attribute_accessor_support;
pub mod constants;
pub mod cli;
pub mod convert;
pub mod codec;
pub mod conventions;
pub mod diagnostics;
pub mod error;
mod failure;
pub mod id;
pub mod io;
pub mod log;
pub mod logging;
pub mod metrics;
pub mod style;
pub mod method_class_key;
pub mod method_parameter;
pub mod named_inheritable_thread_local;
pub mod named_thread_local;
pub mod nested_checked_exception;
pub mod nested_exception_utils;
pub mod nested_runtime_exception;
pub mod collection_factory;
pub mod simple_alias_registry;
pub mod task;
pub mod env;
pub mod ordered;
pub mod order_comparator;
pub mod priority_ordered;
pub mod serialization;
pub mod sorted_properties;
pub mod properties_file;
pub mod retry;
pub mod serializer;
pub mod version;
pub mod time;
pub mod web;
pub mod util;

pub use alias_registry::AliasRegistry;
pub use app_lifecycle_phase::AppLifecyclePhase;
pub use attribute_accessor::AttributeAccessor;
pub use attribute_accessor_support::AttributeAccessorSupport;
pub use simple_alias_registry::SimpleAliasRegistry;
pub use async_runtime::RuntimeType;
#[cfg(feature = "async-runtime")]
pub use async_runtime::TokioRuntime;
pub use method_parameter::MethodParameter;
pub use properties_file::FrameworkProperties;
pub use conventions::{Conventions, PLURAL_SUFFIX};
pub use failure::{BoxError, SharedError};
pub use cli::CliError;
#[cfg(feature = "cli")]
pub use cli::ClapParser;
pub use codec::{ByteArrayDecoder, ByteArrayEncoder, CodecError, Decoder, Encoder, StringDecoder, StringEncoder};
pub use serialization::SerializationError;
#[cfg(feature = "json")]
pub use serialization::JsonCodec;
#[cfg(feature = "xml")]
pub use serialization::XmlCodec;
pub use web::HttpMethod;

/// 当前 Vernal Workspace 发布版本。
///
/// 所有 crate 使用统一 Workspace 版本，因此诊断报告只需要暴露这一份稳定值。
pub const FRAMEWORK_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Vernal 当前承诺的最低 Rust 工具链版本。
///
/// 该值必须与 Workspace 根清单的 `rust-version` 保持一致,并由 MSRV 门禁验证。
pub const MINIMUM_RUST_VERSION: &str = "1.88.0";

/// 返回当前框架成熟度。
///
/// 现有 API 可用于架构评估和实验，但尚未进入稳定语义化版本兼容期。
pub const PROJECT_STATUS: &str = "experimental";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framework_version_is_non_empty() {
        assert!(!FRAMEWORK_VERSION.is_empty());
    }

    #[test]
    fn minimum_rust_version_is_in_semver_shape() {
        let v = MINIMUM_RUST_VERSION;
        assert!(v.len() >= 5, "MSRV 看起来不是 semver: {v}");
        let parts: Vec<&str> = v.split('.').collect();
        assert_eq!(parts.len(), 3, "MSRV 应包含 3 段: {v}");
        for p in &parts {
            assert!(
                p.chars().all(|c| c.is_ascii_digit()),
                "MSRV 段 {p} 应为纯数字"
            );
        }
    }

    #[test]
    fn project_status_is_stable_or_experimental() {
        assert!(
            matches!(PROJECT_STATUS, "experimental" | "alpha" | "beta" | "stable"),
            "PROJECT_STATUS 不是预期枚举值: {PROJECT_STATUS}"
        );
    }

    #[test]
    fn app_lifecycle_phase_reexport_works() {
        let phase = AppLifecyclePhase::Created;
        assert!(!phase.is_active());
        assert_eq!(phase.to_string(), "created");
    }
}
