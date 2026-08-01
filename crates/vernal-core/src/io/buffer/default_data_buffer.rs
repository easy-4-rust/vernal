//! 默认数据缓冲实现。
//!
//! 对标 Spring `org.springframework.core.io.buffer.DefaultDataBuffer`。

use std::sync::{Arc, Mutex};

use super::closeable_data_buffer::CloseableDataBuffer;
use super::data_buffer::DataBuffer;
use super::pooled_data_buffer::PooledDataBuffer;
use super::touchable_data_buffer::TouchableDataBuffer;

/// 共享底层存储。
struct SharedStorage {
    bytes: Mutex<Vec<u8>>,
}

/// 默认数据缓冲实现。
///
/// 对应 Java: org.springframework.core.io.buffer.DefaultDataBuffer
///
/// Spring 语义：Netty 堆缓冲的 vernal 替代——`Vec<u8>` 承载字节；
/// `slice`/`duplicate` 通过 [`Arc`] 共享底层内容并各自维护读写位置
/// （对标 Spring 的共享视图语义），独立缓冲写入时自动扩容，
/// 视图（`slice`/`duplicate`）写入不超过视图容量。
pub struct DefaultDataBuffer {
    storage: Arc<SharedStorage>,
    offset: usize,
    capacity: usize,
    read_position: usize,
    write_position: usize,
    /// 是否独立缓冲（可扩容写入）；视图共享时为 `false`。
    owned: bool,
    released: bool,
}

impl DefaultDataBuffer {
    /// 创建空缓冲（可扩容写入）。
    #[must_use]
    pub fn new() -> Self {
        Self {
            storage: Arc::new(SharedStorage {
                bytes: Mutex::new(Vec::new()),
            }),
            offset: 0,
            capacity: 0,
            read_position: 0,
            write_position: 0,
            owned: true,
            released: false,
        }
    }

    /// 创建带预分配空间的空缓冲（对标 `allocateBuffer(initialCapacity)`）。
    #[must_use]
    pub fn with_preallocated(initial_capacity: usize) -> Self {
        Self {
            storage: Arc::new(SharedStorage {
                bytes: Mutex::new(Vec::with_capacity(initial_capacity)),
            }),
            offset: 0,
            capacity: 0,
            read_position: 0,
            write_position: 0,
            owned: true,
            released: false,
        }
    }

    /// 从字节创建已填充缓冲（对标 `DataBufferFactory#wrap`）。
    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let length = bytes.len();
        Self {
            storage: Arc::new(SharedStorage {
                bytes: Mutex::new(bytes),
            }),
            offset: 0,
            capacity: length,
            read_position: 0,
            write_position: length,
            owned: true,
            released: false,
        }
    }
}

impl Default for DefaultDataBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBuffer for DefaultDataBuffer {
    fn capacity(&self) -> usize {
        self.capacity
    }

    fn read_position(&self) -> usize {
        self.read_position
    }

    fn set_read_position(&mut self, position: usize) {
        self.read_position = position.min(self.write_position);
    }

    fn write_position(&self) -> usize {
        self.write_position
    }

    fn set_write_position(&mut self, position: usize) {
        self.write_position = position.min(self.capacity);
    }

    fn readable(&self) -> usize {
        self.write_position.saturating_sub(self.read_position)
    }

    fn writable(&self) -> usize {
        self.capacity.saturating_sub(self.write_position)
    }

    fn read(&mut self, dst: &mut [u8]) -> usize {
        if self.released || dst.is_empty() {
            return 0;
        }
        let n = self.readable().min(dst.len());
        if n == 0 {
            return 0;
        }
        let bytes = self.storage.bytes.lock().unwrap();
        let start = self.offset + self.read_position;
        dst[..n].copy_from_slice(&bytes[start..start + n]);
        self.read_position += n;
        n
    }

    fn write(&mut self, src: &[u8]) -> usize {
        if self.released || src.is_empty() {
            return 0;
        }
        let mut bytes = self.storage.bytes.lock().unwrap();
        if !self.owned {
            // 视图写入：不超过视图剩余容量（对标 ByteBuffer 容量限制）
            let remaining = self.capacity.saturating_sub(self.write_position);
            let n = remaining.min(src.len());
            if n == 0 {
                return 0;
            }
            let start = self.offset + self.write_position;
            bytes[start..start + n].copy_from_slice(&src[..n]);
            self.write_position += n;
            return n;
        }
        // 独立缓冲：扩容后全量写入（对标 Netty 堆缓冲自动扩容）
        let start = self.offset + self.write_position;
        if start + src.len() > bytes.len() {
            bytes.resize(start + src.len(), 0);
        }
        bytes[start..start + src.len()].copy_from_slice(src);
        self.write_position += src.len();
        self.capacity = bytes.len();
        src.len()
    }

    fn slice(&self, index: usize, length: usize) -> Option<Box<dyn DataBuffer>> {
        if self.released || index + length > self.capacity {
            return None;
        }
        Some(Box::new(DefaultDataBuffer {
            storage: Arc::clone(&self.storage),
            offset: self.offset + index,
            capacity: length,
            read_position: 0,
            write_position: 0,
            owned: false,
            released: false,
        }))
    }

    fn duplicate(&self) -> Box<dyn DataBuffer> {
        // 对标 Spring：共享内容、独立位置，读写位置从源复制；视图不可扩容
        Box::new(DefaultDataBuffer {
            storage: Arc::clone(&self.storage),
            offset: self.offset,
            capacity: self.capacity,
            read_position: self.read_position,
            write_position: self.write_position,
            owned: false,
            released: self.released,
        })
    }

    fn bytes(&self) -> Vec<u8> {
        let bytes = self.storage.bytes.lock().unwrap();
        bytes[self.offset..self.offset + self.capacity].to_vec()
    }
}

impl TouchableDataBuffer for DefaultDataBuffer {}

impl CloseableDataBuffer for DefaultDataBuffer {
    fn close(&mut self) {
        self.released = true;
    }
}

impl PooledDataBuffer for DefaultDataBuffer {
    fn retain(&self) -> &dyn DataBuffer {
        // vernal 无字节池：retain 保持分配状态
        self
    }

    fn release(&mut self) -> bool {
        self.released = true;
        true
    }

    fn is_allocated(&self) -> bool {
        !self.released
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_and_reads_with_positions() {
        // A 类（合同对齐）：对标 Spring 读写位置推进
        let mut buffer = DefaultDataBuffer::new();
        assert_eq!(buffer.write(b"hello"), 5);
        assert_eq!(buffer.write_position(), 5);
        assert_eq!(buffer.readable(), 5);
        let mut dst = [0u8; 5];
        assert_eq!(buffer.read(&mut dst), 5);
        assert_eq!(&dst, b"hello");
        assert_eq!(buffer.read_position(), 5);
        assert_eq!(buffer.readable(), 0);
    }

    #[test]
    fn slice_shares_content_with_own_positions() {
        // A 类（合同对齐）：对标 Spring 视图切片
        let mut buffer = DefaultDataBuffer::from_bytes(b"0123456789".to_vec());
        let mut slice = buffer.slice(2, 4).unwrap();
        assert_eq!(slice.capacity(), 4);
        assert_eq!(slice.bytes(), b"2345");
        // 视图写入回传底层
        assert_eq!(slice.write(b"AB"), 2);
        assert_eq!(buffer.bytes(), b"01AB456789");
    }

    #[test]
    fn slice_out_of_bounds_returns_none() {
        // C 类（错误路径）：越界切片被拒绝
        let buffer = DefaultDataBuffer::from_bytes(b"0123456789".to_vec());
        assert!(buffer.slice(8, 4).is_none());
    }

    #[test]
    fn duplicate_keeps_positions_independently() {
        // B 类（边界行为）：对标 Spring 共享内容独立位置
        let mut buffer = DefaultDataBuffer::from_bytes(b"abcd".to_vec());
        buffer.set_read_position(1);
        let mut dup = buffer.duplicate();
        assert_eq!(dup.read_position(), 1);
        dup.set_read_position(2);
        assert_eq!(buffer.read_position(), 1);
        assert_eq!(dup.bytes(), b"abcd");
    }

    #[test]
    fn owned_buffer_grows_on_write() {
        // D 类（生命周期）：独立缓冲自动扩容
        let mut buffer = DefaultDataBuffer::new();
        buffer.write(&[0u8; 64]);
        assert_eq!(buffer.capacity(), 64);
        buffer.write(&[0u8; 64]);
        assert_eq!(buffer.capacity(), 128);
    }

    #[test]
    fn released_buffer_rejects_read_write() {
        // C 类（错误路径）：释放后读写被拒绝（对标池化缓冲归还语义）
        let mut buffer = DefaultDataBuffer::from_bytes(b"data".to_vec());
        assert!(buffer.is_allocated());
        assert!(buffer.release());
        assert!(!buffer.is_allocated());
        let mut dst = [0u8; 4];
        assert_eq!(buffer.read(&mut dst), 0);
        assert_eq!(buffer.write(b"x"), 0);
    }

    #[test]
    fn close_marks_released() {
        // A 类（合同对齐）：对标 `CloseableDataBuffer#close`
        let mut buffer = DefaultDataBuffer::from_bytes(b"data".to_vec());
        buffer.close();
        assert_eq!(buffer.write(b"x"), 0);
    }

    #[test]
    fn view_write_is_limited_to_view_capacity() {
        // B 类（边界行为）：视图写入不超过视图容量
        let buffer = DefaultDataBuffer::from_bytes(b"0123456789".to_vec());
        let mut slice = buffer.slice(0, 2).unwrap();
        assert_eq!(slice.write(b"ABCD"), 2);
        assert_eq!(slice.bytes(), b"AB");
    }
}
