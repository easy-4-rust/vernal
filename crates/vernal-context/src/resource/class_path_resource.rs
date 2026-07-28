//! Classpath 资源。
//!
//! 对标 Spring `org.springframework.core.io.ClassPathResource`。
//!
//! vernal 中简化为字符串内容（实际生产中应从嵌入资源或文件系统加载）。

use super::resource::Resource;
use std::io;

/// Classpath 资源。
///
/// 对应 Java: org.springframework.core.io.ClassPathResource
#[derive(Debug, Clone)]
pub struct ClassPathResource {
    path: String,
    content: Vec<u8>,
}

impl ClassPathResource {
    /// 从路径与内容创建（用于测试或嵌入式资源场景）。
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            content: Vec::new(),
        }
    }

    /// 从路径与内容字节创建。
    #[must_use]
    pub fn with_content(path: impl Into<String>, content: Vec<u8>) -> Self {
        Self {
            path: path.into(),
            content,
        }
    }

    /// 获取资源路径。
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Resource for ClassPathResource {
    fn exists(&self) -> bool {
        !self.content.is_empty()
    }

    fn is_readable(&self) -> bool {
        !self.content.is_empty()
    }

    fn filename(&self) -> Option<&str> {
        std::path::Path::new(&self.path)
            .file_name()
            .and_then(|n| n.to_str())
    }

    fn description(&self) -> String {
        format!("Classpath 资源 [{}]", self.path)
    }

    fn read_bytes(&self) -> io::Result<Vec<u8>> {
        Ok(self.content.clone())
    }
}
