//! 输出流发布器。
//!
//! 对标 Spring `org.springframework.core.io.buffer.OutputStreamPublisher`。

use std::io::{self, Write};

use super::data_buffer::DataBuffer;
use super::data_buffer_factory::DataBufferFactory;
use super::default_data_buffer_factory::DefaultDataBufferFactory;

/// 输出流发布器。
///
/// 对应 Java: org.springframework.core.io.buffer.OutputStreamPublisher
///
/// Spring 语义：把写入 `OutputStream` 的数据按 `bufferSize` 分块
/// 发布为 `Publisher<DataBuffer>`（供订阅者消费）。vernal 无响应式
/// `Publisher` 抽象，以分块累积 + [`Self::drain_buffers`] 取走模型承担
/// “写入 → 分块缓冲”的语义；订阅与背压由调用方控制。
pub struct OutputStreamPublisher {
    buffer_size: usize,
    chunks: Vec<Box<dyn DataBuffer>>,
    current: Option<Box<dyn DataBuffer>>,
    closed: bool,
}

impl OutputStreamPublisher {
    /// 创建发布器（分块大小默认 4096，对标 Spring 默认 `bufferSize`）。
    #[must_use]
    pub fn new() -> Self {
        Self::with_buffer_size(4096)
    }

    /// 创建指定分块大小的发布器。
    #[must_use]
    pub fn with_buffer_size(buffer_size: usize) -> Self {
        Self {
            buffer_size: buffer_size.max(1),
            chunks: Vec::new(),
            current: None,
            closed: false,
        }
    }

    /// 取走已产出的分块缓冲（对标订阅者消费）。
    #[must_use]
    pub fn drain_buffers(&mut self) -> Vec<Box<dyn DataBuffer>> {
        self.flush_chunk();
        std::mem::take(&mut self.chunks)
    }

    /// 已产出分块数量。
    #[must_use]
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// 是否已关闭。
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// 关闭发布器（对标取消订阅）。
    pub fn close(&mut self) {
        self.flush_chunk();
        self.closed = true;
    }

    /// 把当前分块推入产出列表。
    fn flush_chunk(&mut self) {
        if let Some(chunk) = self.current.take()
            && chunk.write_position() > 0
        {
            self.chunks.push(chunk);
        }
    }
}

impl Default for OutputStreamPublisher {
    fn default() -> Self {
        Self::new()
    }
}

impl Write for OutputStreamPublisher {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.closed {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "发布器已关闭"));
        }
        let factory = DefaultDataBufferFactory::new();
        self.current
            .get_or_insert_with(|| factory.allocate_buffer());
        let mut written = 0;
        while written < buf.len() {
            let chunk = self.current.as_mut().unwrap();
            let remaining = self.buffer_size.saturating_sub(chunk.write_position());
            if remaining == 0 {
                self.flush_chunk();
                self.current = Some(factory.allocate_buffer());
                continue;
            }
            let n = remaining.min(buf.len() - written);
            let n = chunk.write(&buf[written..written + n]);
            written += n;
        }
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_chunk();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_writes_by_buffer_size() {
        // A 类（合同对齐）：对标 Spring 按 bufferSize 分块
        let mut publisher = OutputStreamPublisher::with_buffer_size(4);
        publisher.write_all(b"0123456789").unwrap();
        let buffers = publisher.drain_buffers();
        assert_eq!(buffers.len(), 3);
        assert_eq!(buffers[0].bytes(), b"0123");
        assert_eq!(buffers[1].bytes(), b"4567");
        assert_eq!(buffers[2].bytes(), b"89");
    }

    #[test]
    fn flush_emits_partial_chunk() {
        // B 类（边界行为）：flush 落盘未满分块
        let mut publisher = OutputStreamPublisher::with_buffer_size(16);
        publisher.write_all(b"tiny").unwrap();
        publisher.flush().unwrap();
        assert_eq!(publisher.chunk_count(), 1);
    }

    #[test]
    fn closed_publisher_rejects_writes() {
        // C 类（错误路径）：关闭后写入报错
        let mut publisher = OutputStreamPublisher::new();
        publisher.close();
        assert!(publisher.is_closed());
        assert!(publisher.write(b"x").is_err());
    }
}
