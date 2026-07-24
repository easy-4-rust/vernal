#![forbid(unsafe_code)]
#![doc = "Vernal 各独立内核共享的稳定基础合同。"]

mod failure;

pub use failure::{BoxError, SharedError};

/// 返回当前框架成熟度。
///
/// 现有 API 可用于架构评估和实验，但尚未进入稳定语义化版本兼容期。
pub const PROJECT_STATUS: &str = "experimental";
