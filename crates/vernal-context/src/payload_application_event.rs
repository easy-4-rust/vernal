//! 携带任意负载的应用事件 —— 对标 Spring 7.0
//! `org.springframework.context.PayloadApplicationEvent<T>`。

use std::{
    any::{Any, TypeId},
    sync::Arc,
};

use crate::{ApplicationContextEvent, ApplicationContextEventBase};

/// 携带任意负载的应用事件，对标 Spring 7.0
/// `org.springframework.context.PayloadApplicationEvent<T>`。
///
/// Spring 提供 `ApplicationEventPublisher#publishEvent(Object)` 重载：调用方传入
/// 任意对象时，框架自动包装成 `PayloadApplicationEvent`，使订阅方可以按负载类型
/// 而不是事件类型过滤。vernal 通过 [`crate::EventBus::publish_payload`] 提供
/// 等价能力：调用方传入 source 与 payload，EventBus 创建本结构体并以
/// `PayloadApplicationEvent<T>` 类型广播，订阅方可以按 `T` 订阅。
///
/// 与 Spring 一致：
///
/// - `source` 在构造时冻结，对标 `ApplicationEvent#getSource()`
/// - `timestamp` 取构造时刻的毫秒值，对标 `ApplicationEvent#getTimestamp()`
/// - `payload` 必须非空（Spring JavaDoc `Payload must not be null`）
/// - `payload_type` 通过 [`TypeId`] 暴露，对标 Spring
///   `PayloadApplicationEvent#getResolvableType()`
///
/// # 示例
///
/// ```rust,ignore
/// use std::sync::Arc;
/// use vernal_context::{EventBus, PayloadApplicationEvent};
///
/// # async fn example(bus: EventBus) {
/// // 发布方
/// bus.publish_payload(Arc::new("order-service") as Arc<dyn Any + Send + Sync>, "order-42")
///     .await;
///
/// // 订阅方
/// let mut rx = bus.subscribe::<PayloadApplicationEvent<String>>().await;
/// if let Ok(event) = rx.recv().await {
///     assert_eq!(event.payload(), "order-42");
/// }
/// # }
/// ```
#[derive(Debug)]
pub struct PayloadApplicationEvent<T>
where
    T: Any + Send + Sync + 'static,
{
    base: ApplicationContextEventBase,
    payload: Arc<T>,
    payload_type: TypeId,
}

impl<T> PayloadApplicationEvent<T>
where
    T: Any + Send + Sync + 'static,
{
    /// 创建携带 source / payload 的载荷事件，时间戳取当前系统时钟。
    ///
    /// 对标 Spring `PayloadApplicationEvent(Object source, T payload)`。
    /// payload 与 source 都使用 `Arc` 共享所有权，避免大对象深拷贝。
    #[must_use]
    pub fn new(source: Arc<dyn Any + Send + Sync>, payload: Arc<T>) -> Self {
        let payload_type = TypeId::of::<T>();
        Self {
            base: ApplicationContextEventBase::new(source),
            payload,
            payload_type,
        }
    }

    /// 创建载荷事件，使用指定时间戳（主要供测试与确定性场景使用）。
    ///
    /// 对标 Spring `PayloadApplicationEvent(Object source, T payload, ResolvableType)`：
    /// vernal 不需要 `ResolvableType` 抽象（Rust 的 `TypeId` 已经能区分类型），
    /// 只保留时间戳参数。
    #[must_use]
    pub fn with_timestamp(
        source: Arc<dyn Any + Send + Sync>,
        payload: Arc<T>,
        timestamp: i64,
    ) -> Self {
        let payload_type = TypeId::of::<T>();
        Self {
            base: ApplicationContextEventBase::with_timestamp(source, timestamp),
            payload,
            payload_type,
        }
    }

    /// 返回载荷的共享引用。
    ///
    /// 对标 Spring `PayloadApplicationEvent#getPayload()`。返回 `Arc<T>` 而不是
    /// `&T`，使订阅方可以在异步任务中持有载荷而不需要借用事件对象。
    #[must_use]
    pub fn payload(&self) -> Arc<T> {
        Arc::clone(&self.payload)
    }

    /// 返回载荷的 Rust 类型 ID。
    ///
    /// 对标 Spring `PayloadApplicationEvent#getResolvableType()`：用于类型擦除的
    /// 派发器按载荷类型过滤监听器。vernal 直接使用 [`TypeId`] 而不是
    /// `ResolvableType` 抽象，因为 Rust 的 `TypeId` 已经能在运行期唯一标识类型。
    #[must_use]
    pub const fn payload_type(&self) -> TypeId {
        self.payload_type
    }
}

impl<T> ApplicationContextEvent for PayloadApplicationEvent<T>
where
    T: Any + Send + Sync + 'static,
{
    fn source(&self) -> &dyn Any {
        self.base.source()
    }

    fn timestamp(&self) -> i64 {
        self.base.timestamp()
    }
}

impl<T> Clone for PayloadApplicationEvent<T>
where
    T: Any + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            payload: Arc::clone(&self.payload),
            payload_type: self.payload_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_event_carries_source_payload_and_type() {
        let source: Arc<dyn Any + Send + Sync> = Arc::new("order-service".to_string());
        let payload = Arc::new("order-42".to_string());
        let event = PayloadApplicationEvent::new(source, payload);
        assert_eq!(*event.payload(), "order-42".to_string());
        assert_eq!(event.payload_type(), TypeId::of::<String>());
        // source 可以 downcast 回原始 String。
        let source = event.source().downcast_ref::<String>();
        assert_eq!(source, Some(&"order-service".to_string()));
    }

    #[test]
    fn payload_event_with_timestamp_is_deterministic() {
        let source: Arc<dyn Any + Send + Sync> = Arc::new(0_i32);
        let event =
            PayloadApplicationEvent::with_timestamp(source, Arc::new(42_u64), 1_700_000_000_000);
        assert_eq!(event.timestamp(), 1_700_000_000_000);
        assert_eq!(*event.payload(), 42_u64);
    }

    #[test]
    fn payload_event_clone_preserves_payload_arc() {
        let source: Arc<dyn Any + Send + Sync> = Arc::new("svc");
        let payload = Arc::new("payload-value".to_string());
        let event = PayloadApplicationEvent::new(source, Arc::clone(&payload));
        let cloned = event.clone();
        assert!(Arc::ptr_eq(&event.payload(), &cloned.payload()));
    }
}
