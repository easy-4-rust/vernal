#![forbid(unsafe_code)]
#![doc = "Vernal 各独立内核共享的稳定基础合同。"]

pub mod error;
mod failure;

pub use failure::{BoxError, SharedError};

/// 当前 Vernal Workspace 发布版本。
///
/// 所有 crate 使用统一 Workspace 版本，因此诊断报告只需要暴露这一份稳定值。
pub const FRAMEWORK_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Vernal 当前承诺的最低 Rust 工具链版本。
///
/// 该值必须与 Workspace 根清单的 `rust-version` 保持一致，并由 MSRV 门禁验证。
pub const MINIMUM_RUST_VERSION: &str = "1.85.0";

/// 返回当前框架成熟度。
///
/// 现有 API 可用于架构评估和实验，但尚未进入稳定语义化版本兼容期。
pub const PROJECT_STATUS: &str = "experimental";
