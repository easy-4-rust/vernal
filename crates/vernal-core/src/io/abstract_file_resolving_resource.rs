//! 可解析文件资源的抽象契约。
//!
//! 对标 Spring `org.springframework.core.io.AbstractFileResolvingResource`。

use std::io;
use std::path::PathBuf;

use super::Resource;

/// 可解析文件资源的抽象契约。
///
/// 对应 Java: org.springframework.core.io.AbstractFileResolvingResource
///
/// Spring 语义：当资源底层是文件时提供 `getFile()` 解析；Rust 中以
/// `file_path()` 表达（`None` 表示非文件资源）。
pub trait AbstractFileResolvingResource: Resource {
    /// 若资源底层是文件则返回其路径。
    fn file_path(&self) -> Option<PathBuf>;

    /// 直接读取文件内容（资源非文件时返回错误）。
    ///
    /// # 错误
    ///
    /// 资源不是文件或 IO 失败时返回 [`std::io::Error`]。
    fn read_file_bytes(&self) -> io::Result<Vec<u8>> {
        let Some(path) = self.file_path() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "resource is not file-based",
            ));
        };
        std::fs::read(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::FileSystemResource;

    #[test]
    fn file_backed_resource_resolves_path() {
        // A 类（合同对齐）：对标 Spring `getFile()`
        fn assert_file_resolving<T: AbstractFileResolvingResource>() {}
        assert_file_resolving::<FileSystemResource>();
        let resource = FileSystemResource::new(PathBuf::from("/tmp/abc.properties"));
        let resolved = resource.file_path().unwrap();
        assert_eq!(resolved, PathBuf::from("/tmp/abc.properties"));
    }

    #[test]
    fn missing_file_read_returns_error() {
        // C 类（错误路径）
        let resource = FileSystemResource::new(PathBuf::from("/no/such/xyz"));
        let err = resource.read_file_bytes().unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }
}
