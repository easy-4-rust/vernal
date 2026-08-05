//! 数据缓冲输出流。
//!
//! 对标 Spring `org.springframework.core.io.buffer.DataBufferOutputStream`。

use std::io::{self, Write};

use super::data_buffer::DataBuffer;

/// 数据缓冲输出流。
///
/// 对应 Java: org.springframework.core.io.buffer.DataBufferOutputStream
///
/// Spring 语义：把 `DataBuffer` 暴露为 `OutputStream` 的适配器——
/// 写入推进缓冲的写入位置（缓冲按需扩容）。
pub struct DataBufferOutputStream {
    buffer: Box<dyn DataBuffer>,
    closed: bool,
}

impl DataBufferOutputStream {
    /// 从缓冲创建输出流。
    #[must_use]
    pub fn new(buffer: Box<dyn DataBuffer>) -> Self {
        Self {
            buffer,
            closed: false,
        }
    }

    /// 返回底层缓冲。
    #[must_use]
    pub fn buffer(&self) -> &dyn DataBuffer {
        self.buffer.as_ref()
    }

    /// 是否已关闭。
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// 关闭输出流（对标 `OutputStream#close`）。
    pub fn close(&mut self) {
        self.closed = true;
    }
}

impl Write for DataBufferOutputStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.closed {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "输出流已关闭"));
        }
        let written = self.buffer.write(buf);
        if written == 0 && !buf.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "缓冲拒绝写入（已释放或容量受限）",
            ));
        }
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        // 缓冲无内部冲刷状态
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::buffer::default_data_buffer::DefaultDataBuffer;

    #[test]
    fn writes_into_buffer() {
        // A 类（合同对齐）：对标 Spring 顺序写入
        let buffer = DefaultDataBuffer::new();
        let mut stream = DataBufferOutputStream::new(Box::new(buffer));
        stream.write_all(b"hello").unwrap();
        stream.flush().unwrap();
        assert_eq!(stream.buffer().bytes(), b"hello");
        assert_eq!(stream.buffer().write_position(), 5);
    }

    #[test]
    fn closed_stream_errors_on_write() {
        // C 类（错误路径）：关闭后写入报错
        let buffer = DefaultDataBuffer::new();
        let mut stream = DataBufferOutputStream::new(Box::new(buffer));
        stream.close();
        assert!(stream.is_closed());
        assert!(stream.write(b"x").is_err());
    }

    #[test]
    fn writes_partial_when_buffer_limited() {
        // B 类（边界行为）：视图缓冲容量受限时部分写入
        let inner = DefaultDataBuffer::from_bytes(b"0123456789".to_vec());
        let mut view = inner.slice(0, 2).unwrap();
        let mut stream = DataBufferOutputStream::new(view);
        assert_eq!(stream.write(b"abcd").unwrap(), 2);
    }
}
