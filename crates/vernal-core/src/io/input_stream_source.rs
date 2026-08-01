//! 输入流来源契约。
//!
//! 对标 Spring `org.springframework.core.io.InputStreamSource`。

use std::io;

/// 输入流来源契约。
///
/// 对应 Java: org.springframework.core.io.InputStreamSource
///
/// Spring 语义：所有资源的最基础接口，提供 `getInputStream()`；Rust 中以
/// `read_bytes()` 表达（无显式流生命周期，读取即物化）。
pub trait InputStreamSource: Send + Sync {
    /// 读取资源内容为字节。
    ///
    /// # 错误
    ///
    /// 资源不存在或 IO 失败时返回 [`std::io::Error`]。
    fn read_bytes(&self) -> io::Result<Vec<u8>>;
}

/// 任何 [`Resource`](crate::io::Resource) 天然是输入流来源。
impl<T: crate::io::Resource> InputStreamSource for T {
    fn read_bytes(&self) -> io::Result<Vec<u8>> {
        crate::io::Resource::read_bytes(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_is_input_stream_source() {
        // A 类（合同对齐）：对标 Spring `Resource extends InputStreamSource`
        let source: Box<dyn InputStreamSource> =
            Box::new(crate::io::ByteArrayResource::new("hello".as_bytes().to_vec()));
        assert_eq!(source.read_bytes().unwrap(), b"hello");
    }
}
