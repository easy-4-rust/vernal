//! 路径资源。
//!
//! 对标 Spring `org.springframework.core.io.PathResource`。

use std::io;
use std::path::{Path, PathBuf};

use super::Resource;

/// 路径资源。
///
/// 对应 Java: org.springframework.core.io.PathResource
///
/// Spring 语义：基于 `java.nio.file.Path` 的资源；Rust 中以 `PathBuf` 表达。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathResource {
    path: PathBuf,
}

impl PathResource {
    /// 从路径创建资源。
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// 返回底层路径。
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Resource for PathResource {
    fn exists(&self) -> bool {
        self.path.exists()
    }

    fn is_readable(&self) -> bool {
        self.path.is_file()
    }

    fn filename(&self) -> Option<&str> {
        self.path.file_name().and_then(|n| n.to_str())
    }

    fn description(&self) -> String {
        format!("PathResource [{}]", self.path.display())
    }

    fn read_bytes(&self) -> io::Result<Vec<u8>> {
        std::fs::read(&self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_path_does_not_exist() {
        // B 类（边界行为）：不存在的路径
        let resource = PathResource::new("/no/such/file-xyz");
        assert!(!resource.exists());
        assert!(!resource.is_readable());
    }

    #[test]
    fn description_contains_path() {
        // A 类（合同对齐）：对标 Spring 描述格式
        let resource = PathResource::new("/tmp/x.properties");
        assert!(resource.description().contains("/tmp/x.properties"));
    }

    #[test]
    fn reads_existing_file() {
        // A 类（合同对齐）：对标 Spring 文件读取
        let dir = std::env::temp_dir();
        let path = dir.join("vernal-path-resource-test.txt");
        std::fs::write(&path, b"content").unwrap();
        let resource = PathResource::new(&path);
        assert!(resource.exists());
        assert_eq!(resource.read_bytes().unwrap(), b"content");
        assert_eq!(resource.filename(), Some("vernal-path-resource-test.txt"));
        let _ = std::fs::remove_file(&path);
    }
}
