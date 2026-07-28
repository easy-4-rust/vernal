//! 应用上下文事件基类契约。

use std::{
    any::{Any, TypeId},
    sync::Arc,
    time::SystemTime,
};

/// 应用上下文事件基类契约 —— 对标 Spring 7.0
/// `org.springframework.context.ApplicationEvent`。
///
/// Spring 的 `ApplicationEvent extends EventObject`，提供 `source`（事件源对象）
/// 与 `timestamp`（事件构造时刻的墙上时钟）两个不可变字段。vernal 把基类从
/// 继承改为 trait，事件类型可以选择实现本 trait 暴露 source 与 timestamp；
/// 默认实现由 [`ApplicationContextEventBase`] 提供，封装这两个字段。
///
/// Context-local [`crate::EventBus`] 不强制事件类型实现本 trait —— 简单事件
/// 仍可以是裸 `struct` / `enum`。需要 source / timestamp 的事件，实现方应使用
/// [`ApplicationContextEventBase`] 作为字段或工厂方法构造自身。
///
/// 与 Spring 7.0 一致：timestamp 使用 [`SystemTime::now`] 的毫秒值（对标 Java
/// `System.currentTimeMillis()`）；source 不强制非空，但 Spring JavaDoc 标记
/// "never null"，因此本 trait 的 `source()` 返回 `&dyn Any` 而非 `Option`，由
/// 实现方保证非空。
pub trait ApplicationContextEvent: Any + Send + Sync + 'static {
    /// 返回事件最初发生的对象（事件源）。
    ///
    /// 与 Spring `ApplicationEvent#getSource()` 对应。事件源始终非空，由实现方
    /// 通过 [`ApplicationContextEventBase`] 保证。
    fn source(&self) -> &dyn Any;

    /// 返回事件发生时刻的毫秒时间戳（对标 Spring `ApplicationEvent#getTimestamp()`）。
    ///
    /// 与 Java `System.currentTimeMillis()` 一致：从 UTC 1970-01-01T00:00:00Z
    /// 起的毫秒数。返回 `i64` 而非 `u64`，与 JVM 在 64 位平台上的 `long` 类型
    /// 对齐；负值仅在系统时钟早于 epoch 时出现。
    fn timestamp(&self) -> i64;
}

/// 封装 source / timestamp 的可复用基类，对标 Spring
/// `org.springframework.context.ApplicationEvent` 的字段布局。
///
/// 实现 [`ApplicationContextEvent`] trait 的事件类型可以使用本结构作为字段
/// 载体，避免重复样板代码。两个字段在构造时冻结，运行期不可变。
///
/// # 关于 source 的类型擦除
///
/// 为了让消费方能够通过 `downcast_ref::<T>()` 取回原始 source 类型，本结构
/// 在内部把 source 存为 `Arc<dyn Any + Send + Sync>`，并通过 [`Self::source`]
/// 返回 `&dyn Any`。这与 Spring `ApplicationEvent#getSource()` 返回 `Object`
/// （调用方再 cast）的语义一致。注意 source 必须以**值类型** `Arc<T>`
/// 传入 —— 不能是 `Arc<Arc<T>>` —— 否则 downcast 时会失败。
///
/// # 示例
///
/// ```rust,ignore
/// use vernal_context::{ApplicationContextEvent, ApplicationContextEventBase};
/// use std::sync::Arc;
///
/// struct OrderCreated {
///     base: ApplicationContextEventBase,
///     order_id: u64,
/// }
///
/// impl ApplicationContextEvent for OrderCreated {
///     fn source(&self) -> &dyn std::any::Any { self.base.source() }
///     fn timestamp(&self) -> i64 { self.base.timestamp() }
/// }
///
/// let event = OrderCreated {
///     base: ApplicationContextEventBase::new(Arc::new("order-service")),
///     order_id: 42,
/// };
/// assert_eq!(event.timestamp(), event.base.timestamp());
/// ```
#[derive(Clone)]
pub struct ApplicationContextEventBase {
    source: Arc<dyn Any + Send + Sync>,
    timestamp: i64,
}

impl ApplicationContextEventBase {
    /// 创建事件基类，使用当前系统时钟作为时间戳。
    ///
    /// 与 Spring `ApplicationEvent(Object source)` 构造器一致：source 必须提供，
    /// 时间戳取构造时刻的 `SystemTime::now()` 毫秒值。
    ///
    /// 注意 source 应该是 `Arc<T>`（如 `Arc::new("svc")`），EventBus 与监听器
    /// 才能通过 `downcast_ref::<T>()` 取回原始类型。
    #[must_use]
    pub fn new(source: Arc<dyn Any + Send + Sync>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|duration| i64::try_from(duration.as_millis()).unwrap_or(i64::MAX))
            .unwrap_or(0);
        Self { source, timestamp }
    }

    /// 创建事件基类，使用指定时间戳（主要供测试与确定性场景使用）。
    ///
    /// 对标 Spring `ApplicationEvent(Object source, Clock clock)` 构造器：调用方
    /// 提供时钟，时间戳由时钟读取。vernal 直接接收毫秒值，避免引入 `Clock`
    /// 抽象。
    #[must_use]
    pub fn with_timestamp(source: Arc<dyn Any + Send + Sync>, timestamp: i64) -> Self {
        Self { source, timestamp }
    }

    /// 返回事件源的不可变引用。
    ///
    /// 返回 `&dyn Any` 而非 `&Arc<dyn Any + Send + Sync>`，使调用方可以直接
    /// `downcast_ref::<T>()` 取回原始 source 类型。
    #[must_use]
    pub fn source(&self) -> &dyn Any {
        // 把 `Arc<dyn Any + Send + Sync>` 解引用为 `&(dyn Any + Send + Sync)`，
        // 然后通过 trait upcasting 转换为 `&dyn Any`。
        // Rust 1.86+ 支持 `dyn Trait + Send + Sync` -> `dyn Trait` 的 upcast；
        // 这里通过显式借用 + cast 完成。
        let source: &(dyn Any + Send + Sync) = &*self.source;
        // 通过 Any 的 downcast_ref 在 dyn Any 上工作 —— Send + Sync 在 trait
        // object 上被擦除需要 Rust 1.86+ 的 trait upcasting。
        // 在旧工具链上我们用 Any 的 type_id + 显式 downcast。
        source as &dyn Any
    }

    /// 返回事件时间戳（毫秒）。
    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

impl std::fmt::Debug for ApplicationContextEventBase {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ApplicationContextEventBase")
            .field("source_type", &self.source.type_id())
            .field("timestamp", &self.timestamp)
            .finish_non_exhaustive()
    }
}

/// 返回事件类型的 Rust `TypeId`，用于 `SmartApplicationListener#supportsEventType`
/// 的类型擦除判定（对标 Java `Class<? extends ApplicationEvent>`）。
#[must_use]
pub fn event_type_id<E: Any + Send + Sync + 'static>() -> TypeId {
    TypeId::of::<E>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_base_captures_timestamp_at_construction() {
        let before = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
            .unwrap_or(0);
        let base = ApplicationContextEventBase::new(Arc::new("src"));
        let after = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
            .unwrap_or(0);
        assert!(
            base.timestamp() >= before && base.timestamp() <= after,
            "timestamp {} must be within [{before}, {after}]",
            base.timestamp()
        );
    }

    #[test]
    fn event_base_with_explicit_timestamp() {
        let base = ApplicationContextEventBase::with_timestamp(Arc::new("src"), 1_700_000_000_000);
        assert_eq!(base.timestamp(), 1_700_000_000_000);
    }

    #[test]
    fn event_base_source_can_be_downcast() {
        let source: Arc<dyn Any + Send + Sync> = Arc::new("source-id".to_string());
        let base = ApplicationContextEventBase::new(source);
        let source = base.source().downcast_ref::<String>();
        assert_eq!(source, Some(&"source-id".to_string()));
    }
}
