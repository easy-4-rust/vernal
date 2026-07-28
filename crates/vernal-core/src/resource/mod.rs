//! 资源抽象模块。
//!
//! 对标 Spring `org.springframework.core.io.Resource` 和 `ResourceLoader`。
//!
//! # 与 Spring 的对应关系
//!
//! | Spring | vernal-core |
//! |---|---|
//! | `Resource` interface | `Resource` trait |
//! | `ResourceLoader` interface | `ResourceLoader` trait |
//! | `ClassPathResource` | `ClassPathResource` struct |
//! | `FileSystemResource` | `FileSystemResource` struct |
//! | `ByteArrayResource` | `ByteArrayResource` struct |

use std::path::PathBuf;

/// 资源抽象 trait。
///
/// 对标 Spring `org.springframework.core.io.Resource`。
///
/// # 示例
///
/// ```rust
/// use vernal_core::resource::{Resource, FileSystemResource};
///
/// let resource = FileSystemResource::new("/tmp/test.txt");
/// assert_eq!(resource.filename(), Some("test.txt"));
/// ```
pub trait Resource: Send + Sync {
    /// 资源是否存在。
    fn exists(&self) -> bool;

    /// 是否可读。
    fn is_readable(&self) -> bool;

    /// 获取资源文件名。
    fn filename(&self) -> Option<&str>;

    /// 获取资源描述（用于日志）。
    fn description(&self) -> String;

    /// 读取资源内容为字节数组。
    fn read_bytes(&self) -> std::io::Result<Vec<u8>>;

    /// 读取资源内容为字符串。
    fn read_string(&self) -> std::io::Result<String> {
        let bytes = self.read_bytes()?;
        String::from_utf8(bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

/// 资源加载器 trait。
///
/// 对标 Spring `org.springframework.core.io.ResourceLoader`。
pub trait ResourceLoader: Send + Sync {
    /// 资源类型。
    type Resource: Resource;

    /// 根据位置字符串加载资源。
    ///
    /// 支持的位置格式：
    /// - `classpath:path` - 从类路径加载
    /// - `file:path` - 从文件系统加载
    /// - `url:path` - 从 URL 加载
    fn load(&self, location: &str) -> std::io::Result<Self::Resource>;
}

/// 文件系统资源。
///
/// 对标 Spring `org.springframework.core.io.FileSystemResource`。
#[derive(Debug, Clone)]
pub struct FileSystemResource {
    /// 文件路径
    path: PathBuf,
}

impl FileSystemResource {
    /// 创建新的文件系统资源。
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// 获取文件路径。
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl Resource for FileSystemResource {
    fn exists(&self) -> bool {
        self.path.exists()
    }

    fn is_readable(&self) -> bool {
        self.path.exists() && self.path.is_file()
    }

    fn filename(&self) -> Option<&str> {
        self.path.file_name().and_then(|n| n.to_str())
    }

    fn description(&self) -> String {
        format!("file [{}]", self.path.display())
    }

    fn read_bytes(&self) -> std::io::Result<Vec<u8>> {
        std::fs::read(&self.path)
    }
}

/// 类路径资源。
///
/// 对标 Spring `org.springframework.core.io.ClassPathResource`。
///
/// 注意：Rust 没有 JVM 的类路径概念，此类使用 `include_str!` 或 `include_bytes!`
/// 在编译时嵌入资源。运行时类路径资源需要通过其他机制实现。
#[derive(Debug, Clone)]
pub struct ClassPathResource {
    /// 资源路径
    path: String,
    /// 嵌入的资源内容（编译时确定）
    content: Option<&'static [u8]>,
}

impl ClassPathResource {
    /// 创建新的类路径资源（编译时嵌入）。
    pub fn new(path: impl Into<String>, content: &'static [u8]) -> Self {
        Self {
            path: path.into(),
            content: Some(content),
        }
    }

    /// 创建新的类路径资源（运行时查找）。
    pub fn from_path(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            content: None,
        }
    }

    /// 获取资源路径。
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Resource for ClassPathResource {
    fn exists(&self) -> bool {
        self.content.is_some()
    }

    fn is_readable(&self) -> bool {
        self.content.is_some()
    }

    fn filename(&self) -> Option<&str> {
        self.path.split('/').last()
    }

    fn description(&self) -> String {
        format!("class path resource [{}]", self.path)
    }

    fn read_bytes(&self) -> std::io::Result<Vec<u8>> {
        match self.content {
            Some(bytes) => Ok(bytes.to_vec()),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Class path resource [{}] not found", self.path),
            )),
        }
    }
}

/// 字节数组资源。
///
/// 对标 Spring `org.springframework.core.io.ByteArrayResource`。
#[derive(Debug, Clone)]
pub struct ByteArrayResource {
    /// 资源描述
    description: String,
    /// 字节数组
    bytes: Vec<u8>,
}

impl ByteArrayResource {
    /// 创建新的字节数组资源。
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            description: "Byte array resource".to_string(),
            bytes,
        }
    }

    /// 创建带描述的字节数组资源。
    pub fn with_description(bytes: Vec<u8>, description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            bytes,
        }
    }
}

impl Resource for ByteArrayResource {
    fn exists(&self) -> bool {
        true
    }

    fn is_readable(&self) -> bool {
        true
    }

    fn filename(&self) -> Option<&str> {
        None
    }

    fn description(&self) -> String {
        self.description.clone()
    }

    fn read_bytes(&self) -> std::io::Result<Vec<u8>> {
        Ok(self.bytes.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_system_resource_exists() {
        let resource = FileSystemResource::new("/tmp");
        assert!(resource.exists());
    }

    #[test]
    fn file_system_resource_not_exists() {
        let resource = FileSystemResource::new("/nonexistent/path");
        assert!(!resource.exists());
    }

    #[test]
    fn file_system_resource_filename() {
        let resource = FileSystemResource::new("/tmp/test.txt");
        assert_eq!(resource.filename(), Some("test.txt"));
    }

    #[test]
    fn class_path_resource_with_content() {
        let content = b"hello world";
        let resource = ClassPathResource::new("test.txt", content);
        assert!(resource.exists());
        assert_eq!(resource.read_bytes().unwrap(), content);
    }

    #[test]
    fn class_path_resource_without_content() {
        let resource = ClassPathResource::from_path("nonexistent.txt");
        assert!(!resource.exists());
        assert!(resource.read_bytes().is_err());
    }

    #[test]
    fn byte_array_resource() {
        let bytes = vec![1, 2, 3];
        let resource = ByteArrayResource::new(bytes.clone());
        assert!(resource.exists());
        assert_eq!(resource.read_bytes().unwrap(), bytes);
    }

    #[test]
    fn byte_array_resource_with_description() {
        let resource = ByteArrayResource::with_description(vec![1, 2, 3], "test data");
        assert_eq!(resource.description(), "test data");
    }

    #[test]
    fn resource_read_string() {
        let content = b"hello world";
        let resource = ByteArrayResource::new(content.to_vec());
        assert_eq!(resource.read_string().unwrap(), "hello world");
    }

    #[test]
    fn file_system_resource_is_readable() {
        let resource = FileSystemResource::new("/tmp");
        // /tmp 是目录，不是文件
        assert!(!resource.is_readable());
    }

    #[test]
    fn file_system_resource_description() {
        let resource = FileSystemResource::new("/tmp/test.txt");
        assert!(resource.description().contains("file"));
        assert!(resource.description().contains("test.txt"));
    }

    #[test]
    fn file_system_resource_read_bytes_nonexistent() {
        let resource = FileSystemResource::new("/nonexistent/file.txt");
        assert!(resource.read_bytes().is_err());
    }

    #[test]
    fn class_path_resource_filename() {
        let content = b"hello";
        let resource = ClassPathResource::new("com/example/config.xml", content);
        assert_eq!(resource.filename(), Some("config.xml"));
    }

    #[test]
    fn class_path_resource_description() {
        let content = b"hello";
        let resource = ClassPathResource::new("test.txt", content);
        assert!(resource.description().contains("class path resource"));
        assert!(resource.description().contains("test.txt"));
    }

    #[test]
    fn byte_array_resource_is_readable() {
        let resource = ByteArrayResource::new(vec![1, 2, 3]);
        assert!(resource.is_readable());
    }

    #[test]
    fn byte_array_resource_filename_is_none() {
        let resource = ByteArrayResource::new(vec![1, 2, 3]);
        assert_eq!(resource.filename(), None);
    }

    #[test]
    fn byte_array_resource_read_string() {
        let resource = ByteArrayResource::new(b"hello world".to_vec());
        assert_eq!(resource.read_string().unwrap(), "hello world");
    }

    #[test]
    fn resource_trait_object() {
        let resource: Box<dyn Resource> = Box::new(ByteArrayResource::new(vec![1, 2, 3]));
        assert!(resource.exists());
        assert!(resource.is_readable());
        assert_eq!(resource.read_bytes().unwrap(), vec![1, 2, 3]);
    }
}
