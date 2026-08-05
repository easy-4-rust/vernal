//! 文件系统资源加载器。
//!
//! 对标 Spring `org.springframework.core.io.FileSystemResourceLoader`。

use std::io;
use std::path::PathBuf;

use super::ResourceLoader;
use super::file_system_resource::FileSystemResource;
use super::resource::Resource;

/// 文件系统资源加载器。
///
/// 对应 Java: org.springframework.core.io.FileSystemResourceLoader
///
/// Spring 语义：所有位置都按文件系统路径加载（不识别 `classpath:` 前缀）。
pub struct FileSystemResourceLoader;

impl ResourceLoader for FileSystemResourceLoader {
    fn load(&self, location: &str) -> io::Result<Box<dyn Resource>> {
        Ok(Box::new(FileSystemResource::new(PathBuf::from(location))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_as_file_system_resource() {
        // A 类（合同对齐）：对标 Spring 文件系统加载
        let loader = FileSystemResourceLoader;
        let resource = loader.load("/no/such/file-xyz").unwrap();
        assert!(!resource.exists());
        assert!(resource.description().contains("文件系统资源"));
    }
}
