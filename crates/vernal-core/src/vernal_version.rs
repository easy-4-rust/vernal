//! 框架版本信息门面。
//!
//! 对标 Spring `org.springframework.core.SpringVersion`。

use crate::{FRAMEWORK_VERSION, MINIMUM_RUST_VERSION, PROJECT_STATUS};

/// 框架版本信息门面。
///
/// 对应 Java: org.springframework.core.SpringVersion
///
/// Spring 语义：`SpringVersion.getVersion()` 从实现包元数据返回版本号；
/// vernal 中由 `CARGO_PKG_VERSION` 编译期常量承担。
pub struct VernalVersion;

impl VernalVersion {
    /// 获取框架版本字符串。
    ///
    /// 对应 Java: `SpringVersion.getVersion()`
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

impl std::fmt::Display for VernalVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(Self::full_version_string().as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_non_empty() {
        // A 类（合同对齐）：对标 Spring `getVersion()` 非空
        assert!(!VernalVersion::version().is_empty());
    }

    #[test]
    fn minimum_rust_version_is_semver_shape() {
        // B 类（边界行为）：MSRV 符合语义化版本形态
        let v = VernalVersion::minimum_rust_version();
        let parts: Vec<&str> = v.split('.').collect();
        assert_eq!(parts.len(), 3);
        assert!(parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())));
    }

    #[test]
    fn full_version_string_contains_parts() {
        // A 类（合同对齐）：完整版本串包含版本与状态
        let full = VernalVersion::full_version_string();
        assert!(full.contains(VernalVersion::version()));
        assert!(full.contains(VernalVersion::project_status()));
        assert!(full.contains("MSRV"));
    }

    #[test]
    fn display_uses_full_string() {
        // D 类（重构安全）：Display 委托完整版本串
        let v = VernalVersion;
        assert_eq!(v.to_string(), VernalVersion::full_version_string());
    }
}
