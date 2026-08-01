//! 数据缓冲工厂抽象。
//!
//! 对标 Spring `org.springframework.core.io.buffer.DataBufferFactory`。

use super::data_buffer::DataBuffer;

/// 数据缓冲工厂抽象。
///
/// 对应 Java: org.springframework.core.io.buffer.DataBufferFactory
///
/// Spring 语义：分配与包装 `DataBuffer` 的工厂——空分配、
/// 带初始容量分配与字节包装。
pub trait DataBufferFactory: Send + Sync {
    /// 分配空缓冲。
    fn allocate_buffer(&self) -> Box<dyn DataBuffer>;

    /// 分配带初始容量的空缓冲（对标 `allocateBuffer(int)`）。
    fn allocate_buffer_with_capacity(&self, initial_capacity: usize) -> Box<dyn DataBuffer>;

    /// 包装既有字节为缓冲（对标 `wrap(byte[])`，写入位置为字节长度）。
    fn wrap(&self, bytes: &[u8]) -> Box<dyn DataBuffer>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::buffer::default_data_buffer_factory::DefaultDataBufferFactory;

    #[test]
    fn factory_trait_is_object_safe() {
        // D 类（重构安全）：契约可作 trait 对象使用
        fn accepts(factory: &dyn DataBufferFactory) {
            let _ = factory.allocate_buffer();
        }
        let factory = DefaultDataBufferFactory::new();
        accepts(&factory);
    }
}
