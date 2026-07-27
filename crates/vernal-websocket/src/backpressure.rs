//! WebSocket 发送背压队列。

use std::{sync::Arc, time::Duration};

use tokio::sync::{Mutex, mpsc};

use crate::{WebSocketError, WebSocketMessage};

/// 队列满时的处理策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressurePolicy {
    /// 等待可用容量，超过超时后失败。
    Wait,
    /// 丢弃最新消息。
    DropNewest,
    /// 立即返回错误，并由上层终止连接。
    Terminate,
}

/// 有界 WebSocket 出站队列。
#[derive(Debug, Clone)]
pub struct OutboundQueue {
    sender: mpsc::Sender<WebSocketMessage>,
    receiver: Arc<Mutex<mpsc::Receiver<WebSocketMessage>>>,
    policy: BackpressurePolicy,
    enqueue_timeout: Duration,
}

impl OutboundQueue {
    /// 创建出站队列。
    ///
    /// # Panics
    ///
    /// 当容量为零时 panic；Tokio 有界通道要求正容量。
    #[must_use]
    pub fn new(capacity: usize, policy: BackpressurePolicy, enqueue_timeout: Duration) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
            policy,
            enqueue_timeout,
        }
    }

    /// 将消息加入队列。
    pub async fn enqueue(&self, message: WebSocketMessage) -> Result<(), WebSocketError> {
        match self.policy {
            BackpressurePolicy::Wait => {
                tokio::time::timeout(self.enqueue_timeout, self.sender.send(message))
                    .await
                    .map_err(|_| WebSocketError::Backpressure)?
                    .map_err(|_| WebSocketError::Closed)
            }
            BackpressurePolicy::DropNewest => match self.sender.try_send(message) {
                Ok(()) | Err(mpsc::error::TrySendError::Full(_)) => Ok(()),
                Err(mpsc::error::TrySendError::Closed(_)) => Err(WebSocketError::Closed),
            },
            BackpressurePolicy::Terminate => {
                self.sender.try_send(message).map_err(|error| match error {
                    mpsc::error::TrySendError::Full(_) => WebSocketError::Backpressure,
                    mpsc::error::TrySendError::Closed(_) => WebSocketError::Closed,
                })
            }
        }
    }

    /// 读取下一条消息。
    pub async fn recv(&self) -> Option<WebSocketMessage> {
        self.receiver.lock().await.recv().await
    }

    /// 返回当前剩余容量。
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.sender.capacity()
    }
}
