//! Spring 版本信息。
//!
//! 对标 Spring `org.springframework.core.SpringVersion`。
//!
//! vernal-core 用 `FRAMEWORK_VERSION`、`MINIMUM_RUST_VERSION`、`PROJECT_STATUS` 三个常量
//! 表达等价语义,本模块提供额外的版本格式化方法。

use crate::{FRAMEWORK_VERSION, MINIMUM_RUST_VERSION, PROJECT_STATUS};

/// 版本信息工具。
///
/// 对标 Spring `SpringVersion`。
pub struct SpringVersion;

impl SpringVersion {
    /// 获取框架版本(对标 Spring `SpringVersion.getVersion()`)。
    #[must_use]
    pub fn version() -> &'static str {
        FRAMEWORK_VERSION
    }

    /// 获取最低 Rust 版本。
    #[must_use]
    pub fn minimum_rust_version() -> &'static str {
        MINIMUM_RUST_VERSION
    }

    /// 获取项目状态。
    #[must_use]
    pub fn project_status() -> &'static str {
        PROJECT_STATUS
    }

    /// 获取完整的版本字符串(对标 Spring `SpringVersion.getVersion()` 完整输出)。
    #[must_use]
    pub fn full_version_string() -> String {
        format!(
            "Vernal Framework v{FRAMEWORK_VERSION} ({PROJECT_STATUS}, MSRV {MINIMUM_RUST_VERSION})"
        )
    }
}

impl std::fmt::Display for SpringVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(Self::full_version_string().as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_non_empty() {
        assert!(!SpringVersion::version().is_empty());
    }

    #[test]
    fn minimum_rust_version_is_non_empty() {
        assert!(!SpringVersion::minimum_rust_version().is_empty());
    }

    #[test]
    fn project_status_is_non_empty() {
        assert!(!SpringVersion::project_status().is_empty());
    }

    #[test]
    fn full_version_string_contains_all_parts() {
        let s = SpringVersion::full_version_string();
        assert!(s.contains("Vernal Framework"));
        assert!(s.contains(SpringVersion::version()));
        assert!(s.contains(SpringVersion::project_status()));
        assert!(s.contains(SpringVersion::minimum_rust_version()));
    }

    #[test]
    fn display_outputs_full_version() {
        let s = format!("{}", SpringVersion);
        assert!(s.contains("Vernal Framework"));
    }
}
