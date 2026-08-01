//! 文件系统资源。
//!
//! 对标 Spring `org.springframework.core.io.FileSystemResource`。

use super::resource::Resource;
use std::io;
use std::path::PathBuf;

/// 文件系统资源。
///
/// 对应 Java: org.springframework.core.io.FileSystemResource
#[derive(Debug, Clone)]
pub struct FileSystemResource {
    path: PathBuf,
}

impl FileSystemResource {
    /// 从路径创建资源。
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// 获取路径。
    #[must_use]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl crate::io::AbstractFileResolvingResource for FileSystemResource {
    fn file_path(&self) -> Option<std::path::PathBuf> {
        Some(self.path().clone())
    }
}

impl crate::io::WritableResource for FileSystemResource {
    fn write_bytes(&self, content: &[u8]) -> std::io::Result<()> {
        std::fs::write(self.path(), content)
    }
}

impl Resource for FileSystemResource {
    fn exists(&self) -> bool {
        self.path.exists()
    }

    fn is_readable(&self) -> bool {
        self.path.is_file()
    }

    fn filename(&self) -> Option<&str> {
        self.path.file_name()?.to_str()
    }

    fn description(&self) -> String {
        format!("文件系统资源 [{}]", self.path.display())
    }

    fn read_bytes(&self) -> io::Result<Vec<u8>> {
        std::fs::read(&self.path)
    }
}
