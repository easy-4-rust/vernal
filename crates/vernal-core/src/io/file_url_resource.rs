//! File URL 资源。
//!
//! 对标 Spring `org.springframework.core.io.FileUrlResource`。

use std::io;
use std::path::PathBuf;

use super::Resource;

/// File URL 资源。
///
/// 对应 Java: org.springframework.core.io.FileUrlResource
///
/// Spring 语义：`UrlResource` 的 `file:` 协议特化——把 `file:` URL 解析为
/// 文件系统资源。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileUrlResource {
    path: PathBuf,
    url: String,
}

impl FileUrlResource {
    /// 从 `file:` URL 创建资源。
    ///
    /// 支持 `file:/path` 与 `file:///path` 两种形态。
    #[must_use]
    pub fn new(url: &str) -> Self {
        let stripped = url
            .strip_prefix("file:")
            .unwrap_or(url)
            .strip_prefix("//")
            .unwrap_or_else(|| url.strip_prefix("file:").unwrap_or(url));
        let path = if stripped.starts_with("//") {
            stripped.trim_start_matches('/')
        } else {
            stripped
        };
        Self {
            path: PathBuf::from(path),
            url: url.to_string(),
        }
    }
}

impl Resource for FileUrlResource {
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
        format!("FileUrlResource [{}]", self.url)
    }

    fn read_bytes(&self) -> io::Result<Vec<u8>> {
        std::fs::read(&self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_file_url_with_single_slash() {
        // A 类（合同对齐）：对标 Spring `file:/path`
        let resource = FileUrlResource::new("file:/tmp/x.txt");
        assert!(!resource.exists());
        assert!(resource.description().contains("file:/tmp/x.txt"));
    }

    #[test]
    fn parses_file_url_with_triple_slash() {
        // B 类（边界行为）：对标 Spring `file:///path`
        let resource = FileUrlResource::new("file:///tmp/x.txt");
        assert!(!resource.exists());
    }

    #[test]
    fn reads_existing_file() {
        // A 类（合同对齐）：文件读取
        let dir = std::env::temp_dir();
        let path = dir.join("vernal-file-url-test.txt");
        std::fs::write(&path, b"abc").unwrap();
        let url = format!("file://{}", path.display());
        let resource = FileUrlResource::new(&url);
        assert!(resource.exists());
        assert_eq!(resource.read_bytes().unwrap(), b"abc");
        let _ = std::fs::remove_file(&path);
    }
}
