//! 可关闭数据缓冲契约。
//!
//! 对标 Spring `org.springframework.core.io.buffer.CloseableDataBuffer`。

use super::data_buffer::DataBuffer;

/// 可关闭数据缓冲契约。
///
/// 对应 Java: org.springframework.core.io.buffer.CloseableDataBuffer
///
/// Spring 语义：`DataBuffer` 与 `Closeable` 的合体——显式关闭释放
/// 底层资源。
pub trait CloseableDataBuffer: DataBuffer {
    /// 关闭缓冲（对标 `Closeable#close`）。
    fn close(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::buffer::default_data_buffer::DefaultDataBuffer;

    #[test]
    fn trait_is_object_safe() {
        // D 类（重构安全）：契约可作 trait 对象使用
        fn close_all(buffers: &mut [Box<dyn CloseableDataBuffer>]) {
            for buffer in buffers {
                buffer.close();
            }
        }
        let mut buffers: Vec<Box<dyn CloseableDataBuffer>> =
            vec![Box::new(DefaultDataBuffer::new())];
        close_all(&mut buffers);
    }
}
