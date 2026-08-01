//! 可写资源契约。
//!
//! 对标 Spring `org.springframework.core.io.WritableResource`。

use std::io;

use super::Resource;

/// 可写资源契约。
///
/// 对应 Java: org.springframework.core.io.WritableResource
///
/// Spring 语义：`Resource` 的扩展——支持 `getOutputStream()` 写入；
/// Rust 中以 `write_bytes()` 表达。
pub trait WritableResource: Resource {
    /// 写入资源内容。
    ///
    /// # 错误
    ///
    /// 写入失败时返回 [`std::io::Error`]。
    fn write_bytes(&self, content: &[u8]) -> io::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::FileSystemResource;
    use std::path::PathBuf;

    #[test]
    fn file_system_resource_is_writable() {
        // A 类（合同对齐）：对标 Spring 文件可写资源
        fn assert_writable<T: WritableResource>() {}
        assert_writable::<FileSystemResource>();
        let path = std::env::temp_dir().join("vernal-writable-test.txt");
        let resource = FileSystemResource::new(PathBuf::from(&path));
        resource.write_bytes(b"written").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"written");
        let _ = std::fs::remove_file(&path);
    }
}
