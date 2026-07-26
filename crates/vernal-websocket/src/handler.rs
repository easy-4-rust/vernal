//! WebSocket Handler trait。

/// WebSocket Handler trait。
///
/// 对标 Spring 的 `WebSocketHandler`。
pub trait WebSocketHandler: Send + Sync {
    /// 连接建立时调用。
    fn on_open(&self) {}

    /// 收到消息时调用。
    fn on_message(&self, message: &[u8]) {}

    /// 连接关闭时调用。
    fn on_close(&self) {}

    /// 发生错误时调用。
    fn on_error(&self, error: &str) {}
}
