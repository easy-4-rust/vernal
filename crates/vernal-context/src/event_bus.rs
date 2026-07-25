//! Context 内类型化事件总线对象。

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    num::NonZeroUsize,
    sync::Arc,
};

use tokio::sync::{RwLock, broadcast};

const DEFAULT_CAPACITY: NonZeroUsize = match NonZeroUsize::new(64) {
    Some(capacity) => capacity,
    None => NonZeroUsize::MIN,
};

/// 按事件 Rust 类型隔离的 Tokio 广播总线。
///
/// 每个 `ApplicationContext` 拥有独立总线，不使用进程级静态注册表。事件以
/// `Arc<T>` 广播，多个订阅方共享只读值；慢订阅方遵循 Tokio broadcast 的
/// lagged 语义，由消费方显式处理，不在发布路径无限缓存。
pub struct EventBus {
    capacity: NonZeroUsize,
    senders: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl EventBus {
    /// 创建默认容量为 64 的事件总线。
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    /// 使用每种事件类型的指定环形缓冲容量创建总线。
    #[must_use]
    pub fn with_capacity(capacity: NonZeroUsize) -> Self {
        Self {
            capacity,
            senders: RwLock::new(HashMap::new()),
        }
    }

    /// 订阅一种事件类型。
    pub async fn subscribe<T>(&self) -> broadcast::Receiver<Arc<T>>
    where
        T: Any + Send + Sync,
    {
        let mut senders = self.senders.write().await;
        let sender = senders.entry(TypeId::of::<T>()).or_insert_with(|| {
            let (sender, _) = broadcast::channel::<Arc<T>>(self.capacity.get());
            Box::new(sender)
        });
        if let Some(sender) = sender.downcast_ref::<broadcast::Sender<Arc<T>>>() {
            sender.subscribe()
        } else {
            // TypeId 冲突在安全 Rust 中不应发生；重建通道可避免把内部不变量
            // 转换成用户可触发的 panic。
            let (replacement, receiver) = broadcast::channel(self.capacity.get());
            *sender = Box::new(replacement);
            receiver
        }
    }

    /// 向当前 Context 的所有同类型订阅方发布事件。
    ///
    /// 返回成功接收该事件的订阅方数量；尚无订阅方时返回零。
    pub async fn publish<T>(&self, event: T) -> usize
    where
        T: Any + Send + Sync,
    {
        let mut senders = self.senders.write().await;
        let sender = senders.entry(TypeId::of::<T>()).or_insert_with(|| {
            let (sender, _) = broadcast::channel::<Arc<T>>(self.capacity.get());
            Box::new(sender)
        });
        if let Some(sender) = sender.downcast_ref::<broadcast::Sender<Arc<T>>>() {
            sender.send(Arc::new(event)).unwrap_or_default()
        } else {
            let (replacement, _) = broadcast::channel::<Arc<T>>(self.capacity.get());
            let delivered = replacement.send(Arc::new(event)).unwrap_or_default();
            *sender = Box::new(replacement);
            delivered
        }
    }

    /// 返回指定事件类型的当前订阅方数量。
    ///
    /// 用于诊断和监控。注意：此值是瞬时快照，可能在返回后立即变化。
    pub async fn subscriber_count<T>(&self) -> usize
    where
        T: Any + Send + Sync,
    {
        let senders = self.senders.read().await;
        senders
            .get(&TypeId::of::<T>())
            .and_then(|sender| sender.downcast_ref::<broadcast::Sender<Arc<T>>>())
            .map(|sender| sender.receiver_count())
            .unwrap_or(0)
    }

    /// 返回指定事件类型的通道容量。
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity.get()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
