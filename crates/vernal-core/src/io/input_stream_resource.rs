//! 输入流资源。
//!
//! 对标 Spring `org.springframework.core.io.InputStreamResource`。

use std::io;

use super::Resource;

/// 输入流资源。
///
/// 对应 Java: org.springframework.core.io.InputStreamResource
///
/// Spring 语义：包装内存中的输入流内容；Rust 中以 `Vec<u8>` 承载
/// （读取即物化，对标 Spring 一次性流的语义）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputStreamResource {
    content: Vec<u8>,
    description: String,
}

impl InputStreamResource {
    /// 从字节内容创建资源。
    #[must_use]
    pub fn new(content: Vec<u8>, description: impl Into<String>) -> Self {
        Self {
            content,
            description: description.into(),
        }
    }
}

impl Resource for InputStreamResource {
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

    fn read_bytes(&self) -> io::Result<Vec<u8>> {
        Ok(self.content.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_content() {
        // A 类（合同对齐）：对标 Spring 输入流读取
        let resource = InputStreamResource::new(b"payload".to_vec(), "in-memory");
        assert_eq!(resource.read_bytes().unwrap(), b"payload");
        assert!(resource.exists());
    }

    #[test]
    fn description_is_preserved() {
        // B 类（边界行为）：描述用于诊断
        let resource = InputStreamResource::new(Vec::new(), "request body");
        assert_eq!(resource.description(), "request body");
        assert!(resource.filename().is_none());
    }
}
