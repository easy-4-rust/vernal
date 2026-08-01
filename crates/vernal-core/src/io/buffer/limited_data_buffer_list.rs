//! 受限数据缓冲列表。
//!
//! 对标 Spring `org.springframework.core.io.buffer.LimitedDataBufferList`。

use super::data_buffer::DataBuffer;
use super::data_buffer_limit_exception::DataBufferLimitException;

/// 受限数据缓冲列表。
///
/// 对应 Java: org.springframework.core.io.buffer.LimitedDataBufferList
///
/// Spring 语义：跟踪缓冲总字节数的列表——`add` 超出 `maxTotalSize`
/// 上限时抛出 [`DataBufferLimitException`]；`releaseAndClear` 释放
/// 全部缓冲并清空。
pub struct LimitedDataBufferList {
    buffers: Vec<Box<dyn DataBuffer>>,
    max_total_size: usize,
    total_size: usize,
}

impl LimitedDataBufferList {
    /// 创建带总字节上限的列表。
    #[must_use]
    pub fn new(max_total_size: usize) -> Self {
        Self {
            buffers: Vec::new(),
            max_total_size,
            total_size: 0,
        }
    }

    /// 追加缓冲；超出上限时返回 [`DataBufferLimitException`]。
    pub fn add(&mut self, buffer: Box<dyn DataBuffer>) -> Result<(), DataBufferLimitException> {
        let size = buffer.capacity();
        if self.total_size + size > self.max_total_size {
            return Err(DataBufferLimitException::new(
                self.max_total_size,
                format!("累积大小 {} + {size} 超出上限", self.total_size),
            ));
        }
        self.total_size += size;
        self.buffers.push(buffer);
        Ok(())
    }

    /// 当前缓冲数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.buffers.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }

    /// 当前累积字节数。
    #[must_use]
    pub fn total_size(&self) -> usize {
        self.total_size
    }

    /// 字节上限。
    #[must_use]
    pub fn max_total_size(&self) -> usize {
        self.max_total_size
    }

    /// 释放全部缓冲并清空（对标 Spring `releaseAndClear`）。
    pub fn release_and_clear(&mut self) {
        self.buffers.clear();
        self.total_size = 0;
    }

    /// 取走全部缓冲（释放所有权，不释放缓冲本身）。
    #[must_use]
    pub fn drain_all(&mut self) -> Vec<Box<dyn DataBuffer>> {
        self.total_size = 0;
        std::mem::take(&mut self.buffers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::buffer::default_data_buffer::DefaultDataBuffer;

    #[test]
    fn tracks_total_size() {
        // A 类（合同对齐）：对标 Spring 总量跟踪
        let mut list = LimitedDataBufferList::new(100);
        list.add(Box::new(DefaultDataBuffer::from_bytes(b"0123456789".to_vec())))
            .unwrap();
        list.add(Box::new(DefaultDataBuffer::from_bytes(b"abc".to_vec())))
            .unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list.total_size(), 13);
    }

    #[test]
    fn exceeding_limit_throws() {
        // C 类（错误路径）：超出上限抛 DataBufferLimitException
        let mut list = LimitedDataBufferList::new(5);
        let err = list
            .add(Box::new(DefaultDataBuffer::from_bytes(b"0123456789".to_vec())))
            .unwrap_err();
        assert_eq!(err.max_limit(), 5);
        assert!(list.is_empty());

        list.add(Box::new(DefaultDataBuffer::from_bytes(b"ab".to_vec())))
            .unwrap();
        let err = list
            .add(Box::new(DefaultDataBuffer::from_bytes(b"abcd".to_vec())))
            .unwrap_err();
        assert_eq!(err.max_limit(), 5);
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn release_and_clear_resets_state() {
        // D 类（生命周期）：释放清空后总量归零
        let mut list = LimitedDataBufferList::new(100);
        list.add(Box::new(DefaultDataBuffer::from_bytes(b"hello".to_vec())))
            .unwrap();
        list.release_and_clear();
        assert!(list.is_empty());
        assert_eq!(list.total_size(), 0);
    }

    #[test]
    fn drain_all_returns_buffers() {
        // B 类（边界行为）：取走全部缓冲
        let mut list = LimitedDataBufferList::new(100);
        list.add(Box::new(DefaultDataBuffer::from_bytes(b"ab".to_vec())))
            .unwrap();
        let buffers = list.drain_all();
        assert_eq!(buffers.len(), 1);
        assert!(list.is_empty());
    }
}
