//! FileSystemResourceImpl — 文件系统资源实现。
use crate::resource::Resource;
use std::path::PathBuf;

/// 文件系统资源实现。
#[derive(Clone, Debug)]
pub struct FileSystemResourceImpl {
    pub path: PathBuf,
}
impl FileSystemResourceImpl {
    pub fn new(path: impl Into<PathBuf>) -> Self { Self { path: path.into() } }
    pub fn path(&self) -> &PathBuf { &self.path }
    pub fn filename(&self) -> Option<&str> { self.path.file_name().and_then(|n| n.to_str()) }
}
impl Resource for FileSystemResourceImpl {
    fn exists(&self) -> bool { self.path.exists() }
    fn is_readable(&self) -> bool { self.path.exists() && self.path.is_file() }
    fn description(&self) -> &str { self.path.to_str().unwrap_or("") }
}
