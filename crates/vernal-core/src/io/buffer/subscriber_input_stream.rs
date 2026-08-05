//! 订阅者输入流。
//!
//! 对标 Spring `org.springframework.core.io.buffer.SubscriberInputStream`。

use std::collections::VecDeque;
use std::io::{self, Read};

use super::data_buffer::DataBuffer;

/// 订阅者输入流。
///
/// 对应 Java: org.springframework.core.io.buffer.SubscriberInputStream
///
/// Spring 语义：从 `Publisher<DataBuffer>` 消费数据的阻塞式
/// `InputStream`——缓冲耗尽等待下一块，全部耗尽返回 EOF，关闭即取消订阅。
/// vernal 无响应式 `Publisher`，以预收集的缓冲队列承担消费语义。
pub struct SubscriberInputStream {
    buffers: VecDeque<Box<dyn DataBuffer>>,
    current: Option<Box<dyn DataBuffer>>,
    closed: bool,
}

impl SubscriberInputStream {
    /// 从缓冲队列创建输入流（对标订阅者收到的数据块序列）。
    #[must_use]
    pub fn new(buffers: Vec<Box<dyn DataBuffer>>) -> Self {
        Self {
            buffers: buffers.into(),
            current: None,
            closed: false,
        }
    }

    /// 是否已关闭。
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// 关闭输入流（对标取消订阅）。
    pub fn close(&mut self) {
        self.closed = true;
        self.buffers.clear();
        self.current = None;
    }

    /// 读取单字节（EOF 返回 `None`）。
    pub fn read_byte(&mut self) -> Option<u8> {
        let mut dst = [0u8; 1];
        if self.read(&mut dst).ok()? == 1 {
            Some(dst[0])
        } else {
            None
        }
    }
}

impl Read for SubscriberInputStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.closed {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "输入流已关闭"));
        }
        let mut total = 0;
        while total < buf.len() {
            if self.current.is_none() {
                self.current = self.buffers.pop_front();
            }
            let Some(buffer) = self.current.as_mut() else {
                break; // 全部缓冲耗尽 → EOF
            };
            let n = buffer.read(&mut buf[total..]);
            if n == 0 {
                self.current = None;
                continue;
            }
            total += n;
        }
        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::buffer::default_data_buffer::DefaultDataBuffer;

    fn buffer(bytes: &[u8]) -> Box<dyn DataBuffer> {
        Box::new(DefaultDataBuffer::from_bytes(bytes.to_vec()))
    }

    #[test]
    fn consumes_chunks_in_order() {
        // A 类（合同对齐）：对标 Spring 跨块连续读取
        let mut stream = SubscriberInputStream::new(vec![buffer(b"ab"), buffer(b"cd")]);
        let mut dst = [0u8; 3];
        assert_eq!(stream.read(&mut dst).unwrap(), 3);
        assert_eq!(&dst, b"abc");
        assert_eq!(stream.read_byte(), Some(b'd'));
        assert_eq!(stream.read_byte(), None);
    }

    #[test]
    fn eof_when_buffers_exhausted() {
        // B 类（边界行为）：全部耗尽返回 0（EOF）
        let mut stream = SubscriberInputStream::new(vec![buffer(b"x")]);
        let mut dst = [0u8; 8];
        assert_eq!(stream.read(&mut dst).unwrap(), 1);
        assert_eq!(stream.read(&mut dst).unwrap(), 0);
    }

    #[test]
    fn empty_stream_returns_eof_immediately() {
        // B 类（边界行为）：空队列即 EOF
        let mut stream = SubscriberInputStream::new(Vec::new());
        let mut dst = [0u8; 4];
        assert_eq!(stream.read(&mut dst).unwrap(), 0);
    }

    #[test]
    fn close_cancels_consumption() {
        // C 类（错误路径）：关闭后读取报错（对标取消订阅）
        let mut stream = SubscriberInputStream::new(vec![buffer(b"ab")]);
        stream.close();
        assert!(stream.is_closed());
        let mut dst = [0u8; 1];
        assert!(stream.read(&mut dst).is_err());
        assert_eq!(stream.read_byte(), None);
    }
}
