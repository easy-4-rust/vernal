//! 数据缓冲抽象。
//!
//! 对标 Spring `org.springframework.core.io.buffer.DataBuffer`。

/// 数据缓冲抽象。
///
/// 对应 Java: org.springframework.core.io.buffer.DataBuffer
///
/// Spring 语义：`ByteBuffer` 风格的字节容器——维护读写位置，支持读写、
/// 视图切片（`slice`）与共享内容复制（`duplicate`）。
pub trait DataBuffer: Send + Sync {
    /// 缓冲容量（字节）。
    fn capacity(&self) -> usize;

    /// 读取位置。
    fn read_position(&self) -> usize;

    /// 设置读取位置。
    fn set_read_position(&mut self, position: usize);

    /// 写入位置。
    fn write_position(&self) -> usize;

    /// 设置写入位置。
    fn set_write_position(&mut self, position: usize);

    /// 可读字节数（写入位置 − 读取位置）。
    fn readable(&self) -> usize;

    /// 可写字节数（容量 − 写入位置）。
    fn writable(&self) -> usize;

    /// 从读取位置读入目标切片，推进读取位置；返回实际读取字节数。
    fn read(&mut self, dst: &mut [u8]) -> usize;

    /// 从源切片写入并推进写入位置；返回实际写入字节数。
    fn write(&mut self, src: &[u8]) -> usize;

    /// 返回共享底层内容的视图切片（对标 Spring `slice(index, length)`）。
    ///
    /// 越界或缓冲已释放时返回 `None`。
    fn slice(&self, index: usize, length: usize) -> Option<Box<dyn DataBuffer>>;

    /// 返回共享底层内容、读写位置独立的副本（对标 Spring `duplicate()`）。
    fn duplicate(&self) -> Box<dyn DataBuffer>;

    /// 返回内容快照（对标 `asByteBuffer()` 后的整体读取）。
    fn bytes(&self) -> Vec<u8>;
}
