//! 描述性资源。
//!
//! 对标 Spring `org.springframework.core.io.DescriptiveResource`。

use std::io;

use super::Resource;

/// 描述性资源。
///
/// 对应 Java: org.springframework.core.io.DescriptiveResource
///
/// Spring 语义：仅有描述的资源（`exists=false`），用于错误消息与日志。
pub struct DescriptiveResource {
    description: String,
}

impl DescriptiveResource {
    /// 创建仅含描述的资源。
    #[must_use]
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
        }
    }
}

impl Resource for DescriptiveResource {
    fn exists(&self) -> bool {
        false
    }

    fn is_readable(&self) -> bool {
        false
    }

    fn filename(&self) -> Option<&str> {
        None
    }

    fn description(&self) -> String {
        self.description.clone()
    }

    fn read_bytes(&self) -> io::Result<Vec<u8>> {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            self.description.clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_exists_and_not_readable() {
        // A 类（合同对齐）：对标 Spring 描述性资源语义
        let resource = DescriptiveResource::new("cannot find config");
        assert!(!resource.exists());
        assert!(!resource.is_readable());
    }

    #[test]
    fn read_fails_with_description() {
        // C 类（错误路径）
        let resource = DescriptiveResource::new("missing");
        let err = resource.read_bytes().unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }
}
