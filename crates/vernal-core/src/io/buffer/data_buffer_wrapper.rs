//! 数据缓冲包装器。
//!
//! 对标 Spring `org.springframework.core.io.buffer.DataBufferWrapper`。

use super::data_buffer::DataBuffer;

/// 数据缓冲包装器。
///
/// 对应 Java: org.springframework.core.io.buffer.DataBufferWrapper
///
/// Spring 语义：抽象包装基类——持有委托缓冲并转发全部读写操作，
/// 供装饰器扩展（对标 Spring 各 `*Wrapper` 装饰模式）。
pub struct DataBufferWrapper {
    delegate: Box<dyn DataBuffer>,
}

impl DataBufferWrapper {
    /// 包装一个委托缓冲。
    #[must_use]
    pub fn new(delegate: Box<dyn DataBuffer>) -> Self {
        Self { delegate }
    }

    /// 返回委托缓冲。
    #[must_use]
    pub fn delegate(&self) -> &dyn DataBuffer {
        self.delegate.as_ref()
    }
}

impl DataBuffer for DataBufferWrapper {
    fn capacity(&self) -> usize {
        self.delegate.capacity()
    }

    fn read_position(&self) -> usize {
        self.delegate.read_position()
    }

    fn set_read_position(&mut self, position: usize) {
        self.delegate.set_read_position(position);
    }

    fn write_position(&self) -> usize {
        self.delegate.write_position()
    }

    fn set_write_position(&mut self, position: usize) {
        self.delegate.set_write_position(position);
    }

    fn readable(&self) -> usize {
        self.delegate.readable()
    }

    fn writable(&self) -> usize {
        self.delegate.writable()
    }

    fn read(&mut self, dst: &mut [u8]) -> usize {
        self.delegate.read(dst)
    }

    fn write(&mut self, src: &[u8]) -> usize {
        self.delegate.write(src)
    }

    fn slice(&self, index: usize, length: usize) -> Option<Box<dyn DataBuffer>> {
        self.delegate.slice(index, length)
    }

    fn duplicate(&self) -> Box<dyn DataBuffer> {
        self.delegate.duplicate()
    }

    fn bytes(&self) -> Vec<u8> {
        self.delegate.bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::buffer::default_data_buffer::DefaultDataBuffer;

    #[test]
    fn delegates_all_operations() {
        // A 类（合同对齐）：对标 Spring 委托转发
        let inner = DefaultDataBuffer::from_bytes(b"hello".to_vec());
        let mut wrapper = DataBufferWrapper::new(Box::new(inner));
        assert_eq!(wrapper.capacity(), 5);
        assert_eq!(wrapper.delegate().readable(), 5);
        let mut dst = [0u8; 5];
        assert_eq!(wrapper.read(&mut dst), 5);
        assert_eq!(&dst, b"hello");
        assert_eq!(wrapper.write_position(), 5);
    }

    #[test]
    fn wraps_slice_and_duplicate() {
        // B 类（边界行为）：切片与复制经由委托
        let inner = DefaultDataBuffer::from_bytes(b"0123456789".to_vec());
        let wrapper = DataBufferWrapper::new(Box::new(inner));
        let slice = wrapper.slice(2, 3).unwrap();
        assert_eq!(slice.bytes(), b"234");
        assert_eq!(wrapper.duplicate().bytes(), b"0123456789");
    }
}
