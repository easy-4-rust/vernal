//! FileSystemResource — 文件系统资源。
//!
//! 对应 Java 类：`org.springframework.core.io.FileSystemResource`。
//!
//! 以文件系统路径为后端的 [`Resource`] 实现。

use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::resource::Resource;

/// 文件系统资源。
///
/// 对应 Spring 的 `FileSystemResource`。
///
/// 持有一个 [`PathBuf`]，提供对底层文件的读取与元信息查询。
#[derive(Clone)]
pub struct FileSystemResource {
    /// 底层文件路径。
    path: PathBuf,
}

impl FileSystemResource {
    /// 从路径创建。
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// 从字符串路径创建。
    pub fn from_str(path: impl AsRef<str>) -> Self {
        Self::new(PathBuf::from(path.as_ref()))
    }

    /// 返回底层路径引用。
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 返回底层路径的可变引用。
    pub fn path_mut(&mut self) -> &mut PathBuf {
        &mut self.path
    }
}

impl fmt::Debug for FileSystemResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileSystemResource")
            .field("path", &self.path.display().to_string())
            .finish()
    }
}

impl Resource for FileSystemResource {
    fn exists(&self) -> bool {
        self.path.exists()
    }

    fn is_readable(&self) -> bool {
        self.path.is_file()
    }

    fn is_file(&self) -> bool {
        true
    }

    fn url(&self) -> Option<String> {
        // 构造 file: URL。
        let p = self.path.to_string_lossy();
        Some(format!("file:{p}"))
    }

    fn file_path(&self) -> Option<String> {
        Some(self.path.to_string_lossy().into_owned())
    }

    fn filename(&self) -> Option<String> {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
    }

    fn description(&self) -> String {
        format!("file [{}]", self.path.display())
    }

    fn input_stream(
        &self,
    ) -> Result<Box<dyn Read + Send>, Box<dyn std::error::Error + Send + Sync>> {
        let file = File::open(&self.path)?;
        Ok(Box::new(file))
    }

    fn content_length(&self) -> Option<u64> {
        std::fs::metadata(&self.path).ok().map(|m| m.len())
    }
}

impl PartialEq for FileSystemResource {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for FileSystemResource {}
