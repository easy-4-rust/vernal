//! Context 内类型化事件总线对象。

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

use tokio::sync::{RwLock, broadcast};

use crate::{
    ApplicationEventListener, ApplicationListenerRegistration, EventListenerRegistry, ListenerKey,
};

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
    /// 运行时注册的 listener（对标 Spring `ApplicationEventMulticaster`
    /// 的 listener 注册表）。`HashMap<String, Registration>` 按 listener_id() 索引。
    runtime_listeners: Mutex<HashMap<String, RuntimeRegistration>>,
    /// 类型化 listener 元数据：`Vec<(TypeId, listener_id)>`，顺序匹配 vernal
    /// builder 中 `event_listener()` 的声明顺序（对标 Spring EventListenerFactory
    /// 的隐式顺序）。
    typed_listeners: Mutex<Vec<TypedListenerEntry>>,
}

/// 运行时 listener 注册表中的条目（对标 Spring `ApplicationEventMulticaster`
/// 中的内部 `listenerRetriever`）。
struct RuntimeRegistration {
    type_id: TypeId,
    /// 保留任意 registration 用于直接读取；当前未由 `EventListenerRegistry`
    /// 取出（仅接受注册/移除），但保留扩展点。
    #[allow(dead_code)]
    registration: Box<dyn Any + Send + Sync>,
}

/// 类型化 listener 元数据，用于按 `TypeId` 移除整组 `E` 的订阅。
struct TypedListenerEntry {
    type_id: TypeId,
    listener_id: String,
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
            runtime_listeners: Mutex::new(HashMap::new()),
            typed_listeners: Mutex::new(Vec::new()),
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

    /// 发布携带任意负载的事件，对标 Spring 7.0
    /// `ApplicationEventPublisher#publishEvent(Object payload)`。
    ///
    /// 调用方提供 `source` 与 `payload`；EventBus 包装成
    /// [`crate::PayloadApplicationEvent<T>`] 并按该类型广播。订阅方需要订阅
    /// `PayloadApplicationEvent<T>` 而不是裸 `T`，与 Spring 一致：
    ///
    /// ```rust,ignore
    /// // 发布
    /// bus.publish_payload(source_arc, "order-42").await;
    /// // 订阅
    /// let mut rx = bus.subscribe::<PayloadApplicationEvent<String>>().await;
    /// ```
    ///
    /// 返回成功接收该事件的订阅方数量。
    pub async fn publish_payload<T>(&self, source: Arc<dyn Any + Send + Sync>, payload: T) -> usize
    where
        T: Any + Send + Sync,
    {
        let event: crate::PayloadApplicationEvent<T> =
            crate::PayloadApplicationEvent::new(source, Arc::new(payload));
        self.publish(event).await
    }

    /// 返回指定事件类型的当前订阅方数量。
    ///
    /// 用于诊断和监控。注意：此值是瞬时快照，可能在返回后立即变化。
    pub async fn subscriber_count<T>(&self) -> usize
    where
        T: Any + Send + Sync,
    {
        // 获取读锁后查找该事件类型的 Sender，返回其 Receiver 计数
        let senders = self.senders.read().await;
        senders
            .get(&TypeId::of::<T>())
            .and_then(|sender| sender.downcast_ref::<broadcast::Sender<Arc<T>>>())
            .map(|sender| sender.receiver_count())
            // 该事件类型从未发布或订阅过，返回 0
            .unwrap_or(0)
    }

    /// 返回每种事件类型的通道容量（环形缓冲区大小）。
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity.get()
    }

    /// 异步版本 [`EventListenerRegistry::add_listener`]，与 Spring
    /// `addApplicationListener` 公开方法同名。
    pub async fn add_application_listener<E, L>(
        &self,
        registration: ApplicationListenerRegistration<E, L>,
    ) -> bool
    where
        E: Any + Send + Sync + 'static,
        L: ApplicationEventListener<E> + Send + Sync + 'static,
    {
        <Self as EventListenerRegistry>::add_listener(self, registration)
    }

    /// 异步版本 [`EventListenerRegistry::remove_listener`]，与 Spring
    /// `removeApplicationListener` 公开方法同名。
    pub async fn remove_application_listener(&self, key: &ListenerKey) -> bool {
        <Self as EventListenerRegistry>::remove_listener(self, key)
    }

    /// 异步版本 [`EventListenerRegistry::listener_count`]。
    pub async fn listener_count_async(&self) -> usize {
        <Self as EventListenerRegistry>::listener_count(self)
    }

    /// 返回当前已注册的 listener 总数。
    #[must_use]
    pub fn listener_count(&self) -> usize {
        self.runtime_listeners.lock().expect("poisoned").len()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// `EventListenerRegistry` 适配：当前实现是同步注册（不需要 async 锁 —— 注册
/// 操作仅触碰 `std::sync::Mutex`，不涉及跨任务协作）。
impl EventListenerRegistry for EventBus {
    fn add_listener<E, L>(&self, registration: ApplicationListenerRegistration<E, L>) -> bool
    where
        E: Any + Send + Sync + 'static,
        L: ApplicationEventListener<E> + Send + Sync + 'static,
    {
        let listener_id = registration.listener().listener_id().to_owned();
        let type_id = TypeId::of::<E>();

        let mut runtime_listeners = self
            .runtime_listeners
            .lock()
            .expect("runtime_listeners poisoned");
        if runtime_listeners.contains_key(&listener_id) {
            return false; // 已存在
        }
        runtime_listeners.insert(
            listener_id.clone(),
            RuntimeRegistration {
                type_id,
                registration: Box::new(registration),
            },
        );
        let mut typed = self
            .typed_listeners
            .lock()
            .expect("typed_listeners poisoned");
        typed.push(TypedListenerEntry {
            type_id,
            listener_id,
        });
        true
    }

    fn remove_listener(&self, key: &ListenerKey) -> bool {
        match key {
            ListenerKey::TypeId(type_id) => {
                let mut runtime_listeners = self
                    .runtime_listeners
                    .lock()
                    .expect("runtime_listeners poisoned");
                let mut typed = self
                    .typed_listeners
                    .lock()
                    .expect("typed_listeners poisoned");
                let removed_ids: Vec<String> = runtime_listeners
                    .iter()
                    .filter(|(_, reg)| reg.type_id == *type_id)
                    .map(|(id, _)| id.clone())
                    .collect();
                let removed_count = removed_ids.len();
                for id in &removed_ids {
                    runtime_listeners.remove(id);
                }
                typed.retain(|entry| entry.type_id != *type_id);
                removed_count > 0
            }
            ListenerKey::Named(name) => {
                let mut runtime_listeners = self
                    .runtime_listeners
                    .lock()
                    .expect("runtime_listeners poisoned");
                let mut typed = self
                    .typed_listeners
                    .lock()
                    .expect("typed_listeners poisoned");
                let removed = runtime_listeners.remove(name).is_some();
                typed.retain(|entry| entry.listener_id != *name);
                removed
            }
        }
    }

    fn listener_count(&self) -> usize {
        self.runtime_listeners
            .lock()
            .expect("runtime_listeners poisoned")
            .len()
    }
}
