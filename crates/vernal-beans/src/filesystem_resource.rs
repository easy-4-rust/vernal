//! FileSystemResource — 文件系统资源。
use crate::resource::Resource;
use std::fmt;
use std::path::PathBuf;

/// 文件系统资源。
#[derive(Clone, Debug)]
pub struct FileSystemResource { pub path: PathBuf }
impl FileSystemResource {
    pub fn new(path: impl Into<PathBuf>) -> Self { Self { path: path.into() } }
    pub fn path(&self) -> &PathBuf { &self.path }
    pub fn filename(&self) -> Option<&str> { self.path.file_name().and_then(|n| n.to_str()) }
}
impl Resource for FileSystemResource {
    fn exists(&self) -> bool { self.path.exists() }
    fn is_readable(&self) -> bool { self.path.exists() && self.path.is_file() }
    fn description(&self) -> &str { self.path.to_str().unwrap_or("") }
}
