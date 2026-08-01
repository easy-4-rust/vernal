//! 默认数据缓冲工厂。
//!
//! 对标 Spring `org.springframework.core.io.buffer.DefaultDataBufferFactory`。

use super::data_buffer::DataBuffer;
use super::data_buffer_factory::DataBufferFactory;
use super::default_data_buffer::DefaultDataBuffer;

/// 默认数据缓冲工厂。
///
/// 对应 Java: org.springframework.core.io.buffer.DefaultDataBufferFactory
///
/// Spring 语义：产出 [`DefaultDataBuffer`] 的工厂（对标 Netty 堆缓冲
/// 分配的 vernal 替代）。
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultDataBufferFactory;

impl DefaultDataBufferFactory {
    /// 创建工厂。
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl DataBufferFactory for DefaultDataBufferFactory {
    fn allocate_buffer(&self) -> Box<dyn DataBuffer> {
        Box::new(DefaultDataBuffer::new())
    }

    fn allocate_buffer_with_capacity(&self, initial_capacity: usize) -> Box<dyn DataBuffer> {
        Box::new(DefaultDataBuffer::with_preallocated(initial_capacity))
    }

    fn wrap(&self, bytes: &[u8]) -> Box<dyn DataBuffer> {
        Box::new(DefaultDataBuffer::from_bytes(bytes.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocates_empty_buffer() {
        // A 类（合同对齐）：对标 Spring 空分配
        let factory = DefaultDataBufferFactory::new();
        let buffer = factory.allocate_buffer();
        assert_eq!(buffer.capacity(), 0);
        assert_eq!(buffer.readable(), 0);
    }

    #[test]
    fn allocates_with_initial_capacity() {
        // B 类（边界行为）：初始容量分配可扩容
        let factory = DefaultDataBufferFactory::new();
        let mut buffer = factory.allocate_buffer_with_capacity(16);
        assert_eq!(buffer.write(b"x"), 1);
        assert!(buffer.capacity() >= 1);
    }

    #[test]
    fn wraps_bytes_with_write_position() {
        // A 类（合同对齐）：对标 Spring `wrap(byte[])`
        let factory = DefaultDataBufferFactory::new();
        let buffer = factory.wrap(b"hello");
        assert_eq!(buffer.capacity(), 5);
        assert_eq!(buffer.write_position(), 5);
        assert_eq!(buffer.read_position(), 0);
        assert_eq!(buffer.bytes(), b"hello");
    }
}
