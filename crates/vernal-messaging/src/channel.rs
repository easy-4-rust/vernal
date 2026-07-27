//! 消息通道 trait。

use std::{future::Future, pin::Pin, sync::Arc};

use super::message::Message;

/// 消息通道异步发送返回的 Future。
pub type SendFuture<'a> = Pin<Box<dyn Future<Output = Result<(), MessageError>> + Send + 'a>>;

/// 接收消息返回的 Future。
pub type ReceiveFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Option<Arc<dyn Message>>, MessageError>> + Send + 'a>>;

/// 消息通道 trait。
///
/// 对标 Spring 的 `MessageChannel`，使用 Rust 异步 Future 表达发送/接收。
pub trait MessageChannel: Send + Sync {
    /// 异步发送消息。
    fn send<'a>(&'a self, message: Arc<dyn Message>) -> SendFuture<'a>;

    /// 异步接收消息（非阻塞语义：无消息返回 Ok(None)）。
    fn receive<'a>(&'a self) -> ReceiveFuture<'a> {
        Box::pin(async { Ok(None) })
    }
}

/// 可订阅消息通道 trait。对标 Spring `SubscribableChannel`。
pub trait SubscribableChannel: MessageChannel {
    /// 订阅消息处理器；返回订阅句柄。
    fn subscribe(&self, handler: Arc<dyn MessageHandler>) -> SubscriptionId;

    /// 取消订阅。
    fn unsubscribe(&self, id: SubscriptionId);
}

/// 订阅 ID。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(pub u64);

/// 消息处理器 trait。对标 Spring `MessageHandler`。
pub trait MessageHandler: Send + Sync {
    /// 处理消息。
    fn handle_message<'a>(&'a self, message: Arc<dyn Message>) -> SendFuture<'a>;
}

/// 内存有界消息通道实现。
pub struct InMemoryChannel {
    sender: tokio::sync::mpsc::Sender<Arc<dyn Message>>,
    receiver: tokio::sync::Mutex<tokio::sync::mpsc::Receiver<Arc<dyn Message>>>,
}

impl InMemoryChannel {
    /// 创建容量为 `capacity` 的内存通道。
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(capacity);
        Self {
            sender,
            receiver: tokio::sync::Mutex::new(receiver),
        }
    }
}

impl MessageChannel for InMemoryChannel {
    fn send<'a>(&'a self, message: Arc<dyn Message>) -> SendFuture<'a> {
        Box::pin(async move {
            self.sender.send(message).await.map_err(|_| MessageError {
                message: "channel closed".into(),
            })
        })
    }
    fn receive<'a>(&'a self) -> ReceiveFuture<'a> {
        Box::pin(async move {
            let mut receiver = self.receiver.lock().await;
            Ok(receiver.try_recv().ok())
        })
    }
}

/// 消息错误。
#[derive(Debug, Clone)]
pub struct MessageError {
    /// 错误消息。
    pub message: String,
}

impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "消息错误: {}", self.message)
    }
}

impl std::error::Error for MessageError {}
