//! 池化数据缓冲契约。
//!
//! 对标 Spring `org.springframework.core.io.buffer.PooledDataBuffer`。

use super::data_buffer::DataBuffer;

/// 池化数据缓冲契约。
///
/// 对应 Java: org.springframework.core.io.buffer.PooledDataBuffer
///
/// Spring 语义：可引用计数与归还的 `DataBuffer`——`retain` 增加引用、
/// `release` 归还（计数归零时回收）、`isAllocated` 判断是否仍在分配状态。
/// vernal 无字节池，以释放标记承担计数语义。
pub trait PooledDataBuffer: DataBuffer {
    /// 增加引用计数（vernal 无池化，保持分配状态，对标 retain）。
    fn retain(&self) -> &dyn DataBuffer;

    /// 释放缓冲；返回是否已去分配（对标 `release()` 的 boolean 返回）。
    fn release(&mut self) -> bool;

    /// 缓冲是否仍处于分配状态。
    fn is_allocated(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::buffer::default_data_buffer::DefaultDataBuffer;

    #[test]
    fn trait_is_object_safe() {
        // D 类（重构安全）：契约可作 trait 对象使用
        fn accepts(buffer: &dyn PooledDataBuffer) -> bool {
            buffer.is_allocated()
        }
        let buffer = DefaultDataBuffer::new();
        assert!(accepts(&buffer));
    }
}
