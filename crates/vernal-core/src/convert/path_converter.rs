//! 路径类型转换器。
//!
//! 对标 Spring 的 `StringToPathConverter`（`org.springframework.core.convert.support`）。
//! 提供 `PathBuf` 与字符串之间的双向转换（通过 `Convertible::from_str_value`）。
//!
//! 仅依赖 std，无需任何外部 crate。

use std::path::PathBuf;

use super::{ConversionError, Convertible};

/// 路径转换器。
///
/// 将字符串转换为 [`PathBuf`]，对标 Spring `StringToPathConverter`。
pub struct PathConverter;

impl Convertible for PathBuf {
    fn from_str_value(value: &str) -> Result<Self, ConversionError> {
        if value.is_empty() {
            return Err(ConversionError {
                value: value.to_string(),
                target_type: "PathBuf",
                reason: "路径字符串不能为空".to_string(),
            });
        }
        Ok(PathBuf::from(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn converts_absolute_unix_path() {
        let p = PathBuf::from_str_value("/tmp/foo.txt").unwrap();
        assert_eq!(p, PathBuf::from("/tmp/foo.txt"));
    }

    #[test]
    fn converts_relative_path() {
        let p = PathBuf::from_str_value("./config/app.toml").unwrap();
        assert_eq!(p, PathBuf::from("./config/app.toml"));
    }

    #[test]
    fn converts_windows_path() {
        let p = PathBuf::from_str_value(r"C:\Users\admin\app.toml").unwrap();
        assert_eq!(p, PathBuf::from(r"C:\Users\admin\app.toml"));
    }

    #[test]
    fn rejects_empty_string() {
        let err = PathBuf::from_str_value("").unwrap_err();
        assert_eq!(err.target_type, "PathBuf");
        assert!(err.reason.contains("不能为空"));
    }

    #[test]
    fn preserves_special_chars() {
        let p = PathBuf::from_str_value("/tmp/with spaces/and-dashes/file.log").unwrap();
        assert!(p.to_str().unwrap().contains("with spaces"));
        assert!(p.to_str().unwrap().contains("and-dashes"));
    }

    #[test]
    fn path_conversion_via_conversion_service() {
        let p: PathBuf = super::super::ConversionService::convert("/etc/hosts").unwrap();
        assert_eq!(p, PathBuf::from("/etc/hosts"));
    }
}
