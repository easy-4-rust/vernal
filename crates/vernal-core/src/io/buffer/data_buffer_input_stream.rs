//! 数据缓冲输入流。
//!
//! 对标 Spring `org.springframework.core.io.buffer.DataBufferInputStream`。

use std::io::{self, Read};

use super::data_buffer::DataBuffer;

/// 数据缓冲输入流。
///
/// 对应 Java: org.springframework.core.io.buffer.DataBufferInputStream
///
/// Spring 语义：把 `DataBuffer` 暴露为 `InputStream` 的适配器——
/// 读取推进缓冲的读取位置，缓冲耗尽返回 EOF。
pub struct DataBufferInputStream {
    buffer: Box<dyn DataBuffer>,
    closed: bool,
}

impl DataBufferInputStream {
    /// 从缓冲创建输入流。
    #[must_use]
    pub fn new(buffer: Box<dyn DataBuffer>) -> Self {
        Self {
            buffer,
            closed: false,
        }
    }

    /// 读取单字节（对标 `InputStream#read()`，EOF 返回 `None`）。
    pub fn read_byte(&mut self) -> Option<u8> {
        if self.closed {
            return None;
        }
        let mut dst = [0u8; 1];
        if self.buffer.read(&mut dst) == 1 {
            Some(dst[0])
        } else {
            None
        }
    }

    /// 剩余可读字节数（对标 `InputStream#available()`）。
    #[must_use]
    pub fn available(&self) -> usize {
        self.buffer.readable()
    }

    /// 是否已关闭。
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// 关闭输入流。
    pub fn close(&mut self) {
        self.closed = true;
    }
}

impl Read for DataBufferInputStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.closed {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "输入流已关闭",
            ));
        }
        Ok(self.buffer.read(buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::buffer::default_data_buffer::DefaultDataBuffer;

    #[test]
    fn reads_sequentially_until_eof() {
        // A 类（合同对齐）：对标 Spring 顺序读取至 EOF
        let buffer = DefaultDataBuffer::from_bytes(b"abc".to_vec());
        let mut stream = DataBufferInputStream::new(Box::new(buffer));
        assert_eq!(stream.read_byte(), Some(b'a'));
        assert_eq!(stream.available(), 2);
        let mut dst = [0u8; 2];
        assert_eq!(stream.read(&mut dst).unwrap(), 2);
        assert_eq!(&dst, b"bc");
        assert_eq!(stream.read_byte(), None);
    }

    #[test]
    fn closed_stream_errors_on_read() {
        // C 类（错误路径）：关闭后读取报错（对标 Closeable 语义）
        let buffer = DefaultDataBuffer::from_bytes(b"abc".to_vec());
        let mut stream = DataBufferInputStream::new(Box::new(buffer));
        stream.close();
        assert!(stream.is_closed());
        let mut dst = [0u8; 1];
        assert!(stream.read(&mut dst).is_err());
        assert_eq!(stream.read_byte(), None);
    }

    #[test]
    fn implements_std_read_trait() {
        // D 类（重构安全）：接入 std::io::Read 生态
        fn copy_all<R: Read>(reader: &mut R) -> Vec<u8> {
            let mut out = Vec::new();
            reader.read_to_end(&mut out).unwrap();
            out
        }
        let buffer = DefaultDataBuffer::from_bytes(b"hello".to_vec());
        let mut stream = DataBufferInputStream::new(Box::new(buffer));
        assert_eq!(copy_all(&mut stream), b"hello");
    }
}
