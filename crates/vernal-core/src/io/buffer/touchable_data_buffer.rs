//! 可触碰数据缓冲契约。
//!
//! 对标 Spring `org.springframework.core.io.buffer.TouchableDataBuffer`。

use super::data_buffer::DataBuffer;

/// 可触碰数据缓冲契约。
///
/// 对应 Java: org.springframework.core.io.buffer.TouchableDataBuffer
///
/// Spring 语义：支持 `touch()` 追踪调试的 `DataBuffer`——供内存追踪器
/// 标记缓冲访问（如 Netty 泄漏追踪）。vernal 中为无操作默认实现。
pub trait TouchableDataBuffer: DataBuffer {
    /// 标记触碰（默认无操作）。
    fn touch(&self) -> &Self {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::buffer::default_data_buffer::DefaultDataBuffer;

    #[test]
    fn touch_returns_self() {
        // A 类（合同对齐）：对标 Spring 追踪标记
        fn assert_touchable<T: TouchableDataBuffer>() {}
        assert_touchable::<DefaultDataBuffer>();
        let buffer = DefaultDataBuffer::new();
        assert_eq!(buffer.touch().capacity(), 0);
    }
}
