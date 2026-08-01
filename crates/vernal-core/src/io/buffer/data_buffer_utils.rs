//! 数据缓冲工具。
//!
//! 对标 Spring `org.springframework.core.io.buffer.DataBufferUtils`。

use std::io::{self, Read, Write};

use crate::io::Resource;

use super::data_buffer::DataBuffer;
use super::data_buffer_factory::DataBufferFactory;
use super::default_data_buffer_factory::DefaultDataBufferFactory;

/// 数据缓冲工具（静态辅助函数）。
///
/// 对应 Java: org.springframework.core.io.buffer.DataBufferUtils
///
/// Spring 语义：缓冲的读写、释放与资源桥接工具——
/// `read` 把资源读入缓冲、`write` 把缓冲写入写出器、
/// `release` 归还池化缓冲。
pub struct DataBufferUtils;

impl DataBufferUtils {
    /// 读取资源全部内容到新缓冲（对标 Spring 从资源读取数据流）。
    ///
    /// # 错误
    ///
    /// 资源读取失败时返回 [`std::io::Error`]。
    pub fn read(resource: &dyn Resource) -> io::Result<Box<dyn DataBuffer>> {
        let factory = DefaultDataBufferFactory::new();
        let mut buffer = factory.allocate_buffer();
        let mut reader = resource.read_bytes()?;
        buffer.write(&reader);
        Ok(buffer)
    }

    /// 把缓冲可读内容写入写出器（对标 Spring `write(DataBuffer, OutputStream)`）。
    ///
    /// # 错误
    ///
    /// 写出失败时返回 [`std::io::Error`]。
    pub fn write(buffer: &mut dyn DataBuffer, writer: &mut dyn Write) -> io::Result<()> {
        let mut chunk = [0u8; 8192];
        loop {
            let n = buffer.read(&mut chunk);
            if n == 0 {
                return Ok(());
            }
            writer.write_all(&chunk[..n])?;
        }
    }

    /// 读取输入流全部内容到新缓冲（对标 Spring 从 `InputStream` 读取）。
    ///
    /// # 错误
    ///
    /// 读取失败时返回 [`std::io::Error`]。
    pub fn read_input(reader: &mut dyn Read) -> io::Result<Box<dyn DataBuffer>> {
        let factory = DefaultDataBufferFactory::new();
        let mut buffer = factory.allocate_buffer();
        let mut chunk = [0u8; 8192];
        loop {
            let n = reader.read(&mut chunk)?;
            if n == 0 {
                return Ok(buffer);
            }
            buffer.write(&chunk[..n]);
        }
    }

    /// 释放缓冲（对标 Spring `release(DataBuffer)`）。
    ///
    /// vernal 无字节池：缓冲随所有权自动回收；调用方显式释放后
    /// 不得再使用该缓冲。
    pub fn release(buffer: Box<dyn DataBuffer>) {
        drop(buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::buffer::default_data_buffer::DefaultDataBuffer;
    use crate::io::ByteArrayResource;

    #[test]
    fn reads_resource_into_buffer() {
        // A 类（合同对齐）：对标 Spring 资源→缓冲
        let resource = ByteArrayResource::new(b"payload".to_vec());
        let buffer = DataBufferUtils::read(&resource).unwrap();
        assert_eq!(buffer.bytes(), b"payload");
    }

    #[test]
    fn writes_buffer_to_writer() {
        // A 类（合同对齐）：对标 Spring 缓冲→输出流
        let mut buffer = DefaultDataBuffer::from_bytes(b"hello".to_vec());
        let mut out = Vec::new();
        DataBufferUtils::write(&mut buffer, &mut out).unwrap();
        assert_eq!(out, b"hello");
        // 读取推进位置后不再重复输出
        assert_eq!(buffer.readable(), 0);
    }

    #[test]
    fn reads_input_stream_into_buffer() {
        // B 类（边界行为）：输入流→缓冲
        let mut reader = std::io::Cursor::new(b"data".to_vec());
        let buffer = DataBufferUtils::read_input(&mut reader).unwrap();
        assert_eq!(buffer.bytes(), b"data");
    }

    #[test]
    fn release_drops_buffer() {
        // D 类（生命周期）：释放后缓冲不可再用
        let buffer: Box<dyn DataBuffer> = Box::new(DefaultDataBuffer::from_bytes(b"x".to_vec()));
        DataBufferUtils::release(buffer);
    }
}
