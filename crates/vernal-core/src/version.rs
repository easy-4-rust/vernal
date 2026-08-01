//! 框架版本信息工具。
//!
//! 提供版本号、最低 Rust 版本、项目状态等元数据的格式化方法。
//!
//! 设计说明：核心常量 [`FRAMEWORK_VERSION`] / [`MINIMUM_RUST_VERSION`] /
//! [`PROJECT_STATUS`] 直接定义在 [`crate`] 根模块，本结构体只提供组合格式化方法。

use crate::{FRAMEWORK_VERSION, MINIMUM_RUST_VERSION, PROJECT_STATUS};

/// 框架版本信息工具。
///
/// 提供版本号格式化方法。常量本身定义在 `crate::FRAMEWORK_VERSION` 等。
pub struct FrameworkVersion;

impl FrameworkVersion {
    /// 获取框架版本字符串。
    ///
    /// 对应语义（Spring 迁移）：`SpringVersion.getVersion()`
    #[must_use]
    pub fn version() -> &'static str {
        FRAMEWORK_VERSION
    }

    /// 获取最低 Rust 版本（MSRV）。
    #[must_use]
    pub fn minimum_rust_version() -> &'static str {
        MINIMUM_RUST_VERSION
    }

    /// 获取项目状态。
    #[must_use]
    pub fn project_status() -> &'static str {
        PROJECT_STATUS
    }

    /// 获取完整的版本字符串。
    #[must_use]
    pub fn full_version_string() -> String {
        format!(
            "Vernal Framework v{FRAMEWORK_VERSION} ({PROJECT_STATUS}, MSRV {MINIMUM_RUST_VERSION})"
        )
    }
}

impl std::fmt::Display for FrameworkVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(Self::full_version_string().as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_non_empty() {
        assert!(!FrameworkVersion::version().is_empty());
    }

    #[test]
    fn minimum_rust_version_is_non_empty() {
        assert!(!FrameworkVersion::minimum_rust_version().is_empty());
    }

    #[test]
    fn project_status_is_non_empty() {
        assert!(!FrameworkVersion::project_status().is_empty());
    }

    #[test]
    fn full_version_string_contains_all_parts() {
        let s = FrameworkVersion::full_version_string();
        assert!(s.contains("Vernal Framework"));
        assert!(s.contains(FrameworkVersion::version()));
        assert!(s.contains(FrameworkVersion::project_status()));
        assert!(s.contains(FrameworkVersion::minimum_rust_version()));
    }

    #[test]
    fn display_outputs_full_version() {
        let s = format!("{FrameworkVersion}");
        assert!(s.contains("Vernal Framework"));
    }
}
