#![allow(unused)] // deny(unsafe_code) 移除：Rust 2024 edition 中 set_var 需要 unsafe 块
#![doc = "Vernal 各独立内核共享的稳定基础合同。"]

pub mod app_lifecycle_phase;
pub mod constants;
pub mod convert;
pub mod conventions;
pub mod diagnostics;
pub mod error;
mod failure;
pub mod id;
pub mod logging;
pub mod method_parameter;
pub mod task;
pub mod environment;
pub mod ordered;
pub mod sorted_properties;
pub mod spring_properties;
pub mod spring_version;
pub mod time;
pub mod util;

pub use app_lifecycle_phase::AppLifecyclePhase;
pub use method_parameter::MethodParameter;
pub use spring_properties::SpringProperties;
pub use conventions::{Conventions, PLURAL_SUFFIX};
pub use failure::{BoxError, SharedError};

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
