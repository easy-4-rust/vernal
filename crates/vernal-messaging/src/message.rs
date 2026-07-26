//! 消息 trait。

/// 消息 trait。
///
/// 对标 Spring 的 `Message<T>`。
pub trait Message: Send + Sync {
    /// 消息 ID。
    fn id(&self) -> &str;

    /// 消息负载。
    fn payload(&self) -> &[u8];
}
