//! 应用监听器注册对象 —— Spring `ApplicationListener<?>` 句柄。
//!
//! 对标 `ConfigurableApplicationContext#addApplicationListener` 的运行时
//! 注册路径。Spring 在 refresh 前调用该方法追加监听器；vernal 由于生命周期阶段
//! 在 start 阶段订阅 `EventBus`，运行时添加的监听器必须重新走 `consume` 启动循环。
//!
//! 本模块提供：
//! - `ApplicationListenerRegistration<E>` —— 显式持有泛型 E 的 `Any + Send + Sync` 监听器句柄
//! - `EventListenerRegistry` trait —— 动态添加/移除监听器
//! - `ListenerKey` 标识符（对标 `SmartApplicationListener#getListenerId`）

use std::{any::TypeId, sync::Arc};

use crate::ApplicationEventListener;

/// 运行时监听器注册的唯一标识符（对标 `SmartApplicationListener#getListenerId`）。
///
/// - `ListenerKey::TypeId`：使用事件的 Rust `TypeId`（即泛型 `E` 的 `TypeId`）。
///   此模式适用于不关心具体监听器身份的批量移除（如 `remove_listeners_for::<E>()`）。
/// - `ListenerKey::Named`：调用方提供的字符串标识符，与 Spring `getListenerId()` 返回值对齐。
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ListenerKey {
    /// 按事件 Rust 类型移除。
    TypeId(TypeId),
    /// 按显式字符串标识符移除（对标 Spring `ApplicationEventMulticaster#removeApplicationListeners`）。
    Named(String),
}

impl ListenerKey {
    /// 构造按事件类型匹配的 Key。
    pub fn for_event<E: 'static>() -> Self {
        Self::TypeId(TypeId::of::<E>())
    }

    /// 构造按字符串标识符匹配的 Key。
    #[must_use]
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
    }
}

/// 运行时注册的监听器句柄。
///
/// 对标 `ApplicationListener<?>` 注册句柄 + `SmartApplicationListener#getListenerId`。
/// 调用方持有 `Arc<L>` 即可在 `EventListenerRegistry::add_listener` / `remove_listener`
/// 中管理生命周期。
pub struct ApplicationListenerRegistration<E, L>
where
    E: std::any::Any + Send + Sync + 'static,
    L: ApplicationEventListener<E> + Send + Sync + 'static,
{
    listener: Arc<L>,
    marker: std::marker::PhantomData<fn() -> E>,
}

impl<E, L> ApplicationListenerRegistration<E, L>
where
    E: std::any::Any + Send + Sync + 'static,
    L: ApplicationEventListener<E> + Send + Sync + 'static,
{
    /// 由监听器组件实例创建注册句柄。
    pub fn new(listener: Arc<L>) -> Self {
        Self {
            listener,
            marker: std::marker::PhantomData,
        }
    }

    /// 返回内部监听器组件的共享所有权。
    pub fn listener(&self) -> Arc<L> {
        Arc::clone(&self.listener)
    }
}

/// 运行时事件监听器注册表 —— 对标
/// `ConfigurableApplicationContext#addApplicationListener` /
/// `#removeApplicationListener`。
///
/// 不直接暴露给业务方使用：vernal 把注册表嵌入 `EventBus`，调用方通过
/// [`crate::ApplicationContext`] 间接调用。
pub trait EventListenerRegistry: Send + Sync + 'static {
    /// 注册一个新的事件监听器（对标 Spring `addApplicationListener`）。
    ///
    /// 监听器的 `listener_id()` 会被显式记住；如果已存在同名 listener，方法返回
    /// `false`，已有的 listener 不会被替换（与 Spring `addApplicationListener` 的
    /// 重复检测行为一致 —— Spring 内部通过 `linkedHashSet` 取消重复添加）。
    fn add_listener<E, L>(&self, registration: ApplicationListenerRegistration<E, L>) -> bool
    where
        E: std::any::Any + Send + Sync + 'static,
        L: ApplicationEventListener<E> + Send + Sync + 'static;

    /// 按 key 移除已注册的监听器（对标 Spring `removeApplicationListener` /
    /// `removeApplicationListeners`）。
    ///
    /// 返回是否实际移除了某个 listener。
    fn remove_listener(&self, key: &ListenerKey) -> bool;

    /// 返回当前已注册的事件监听器数量。
    fn listener_count(&self) -> usize;
}
