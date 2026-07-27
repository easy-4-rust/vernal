//! `IoC` 托管应用事件监听器契约。

use std::{any::Any, error::Error, future::Future, sync::Arc};

/// 处理一种 Context-local 强类型应用事件的组件契约。
///
/// 实现类型必须作为 Singleton 注册到 Vernal IoC，并通过
/// [`crate::VernalApplicationBuilder::event_listener`] 或应用模块声明监听关系。
/// Vernal 在 `refresh()` 阶段解析同一个组件实例，先建立 Tokio broadcast 订阅，
/// 再初始化任何生命周期组件，因此初始化阶段发布的事件也不会错过。
///
/// 本 Trait 保持泛型而不支持 `dyn ApplicationEventListener<_>`。类型擦除只发生在
/// 应用构建阶段的启动声明中，事件分发热路径仍调用具体实现，消费方可以让同一组件
/// 分别实现多个事件类型的监听契约。
///
/// # SmartApplicationListener 等价钩子
///
/// 本 trait 同时合并了 Spring `SmartApplicationListener` 与
/// `GenericApplicationListener` 的元数据钩子：
///
/// - [`Self::supports_event_type`]：事件类型过滤，默认 `true`（对标
///   `SmartApplicationListener#supportsEventType(Class<? extends ApplicationEvent>)`）
/// - [`Self::supports_source`]：事件源类型过滤，默认 `true`（对标
///   `SmartApplicationListener#supportsSourceType(Class<?>)`）
/// - [`Self::listener_id`]：监听器稳定标识，默认空字符串（对标
///   `SmartApplicationListener#getListenerId()`，since 5.3.5）
/// - [`Self::order`]：同事件类型多个监听器的派发顺序，默认 `i32::MAX`（对标
///   `SmartApplicationListener#getOrder()` / `Ordered.LOWEST_PRECEDENCE`）
///
/// 默认实现与 Spring 7.0 完全一致：未覆盖时所有钩子都返回最宽松结果（接受全部
/// 事件与源、最低优先级、空标识）。EventBus 在派发时会按 `order` 升序调用，
/// 并在 `supports_event_type` / `supports_source` 返回 `false` 时跳过该监听器。
pub trait ApplicationEventListener<E>: Send + Sync + 'static
where
    E: Any + Send + Sync + 'static,
{
    /// 监听器返回的结构化错误类型。
    type Error: Error + Send + Sync + 'static;

    /// 异步处理一个由所有同类型订阅方共享的只读事件。
    ///
    /// 返回错误会被视为关键后台任务失败：Vernal 保存原始错误链、取消应用并通过
    /// Context 关闭结果暴露失败。实现方不应把凭证、Token 或请求正文写入
    /// [`std::fmt::Display`] 输出。
    fn on_event(&self, event: Arc<E>) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// 返回低基数、无敏感数据的静态监听器名称。
    ///
    /// 默认使用实现类型全名；覆盖值会作为受管 Tokio 任务身份进入错误诊断。
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// 判断该监听器是否支持指定事件类型。
    ///
    /// 对标 Spring `SmartApplicationListener#supportsEventType`。默认 `true`：vernal
    /// 已经通过泛型 `E` 在订阅阶段过滤事件类型，运行期再次过滤主要用于"同一
    /// 组件实现多个监听器 trait，但只对其中部分类型感兴趣"的场景。需要类型过滤
    /// 的实现可以覆盖本方法返回 `false` 让 EventBus 跳过本次派发。
    ///
    /// 参数是事件的 `TypeId`（对标 Java `Class<? extends ApplicationEvent>`）。
    /// 实现方可以使用 [`std::any::TypeId::of::<E>()`] 与传入值比较。
    #[must_use]
    fn supports_event_type(&self, _event_type: std::any::TypeId) -> bool {
        true
    }

    /// 判断该监听器是否支持指定事件源类型。
    ///
    /// 对标 Spring `SmartApplicationListener#supportsSourceType`。默认 `true`，
    /// 与 Spring 7.0 一致。需要按源过滤的实现（例如只接受来自
    /// `OrderService` 的事件）可以覆盖本方法。
    ///
    /// 参数是事件源的 `TypeId`；如果事件未携带源信息则传入 `None`，实现方
    /// 可以选择接受或拒绝。
    #[must_use]
    fn supports_source(&self, _source_type: Option<std::any::TypeId>) -> bool {
        true
    }

    /// 返回监听器稳定标识。
    ///
    /// 对标 Spring `SmartApplicationListener#getListenerId()`（since 5.3.5）。
    /// 默认空字符串。需要通过 `EventBus::remove_listener` 等运行期管理接口
    /// 精确移除监听器的应用可以覆盖本方法返回稳定字符串。
    #[must_use]
    fn listener_id(&self) -> &'static str {
        ""
    }

    /// 返回同事件类型多个监听器的派发顺序值。
    ///
    /// 对标 Spring `SmartApplicationListener#getOrder()` 与
    /// `org.springframework.core.Ordered#getOrder()`：值越小越先派发。
    /// 默认 `i32::MAX`，对应 Spring `Ordered.LOWEST_PRECEDENCE`
    /// （`Integer.MAX_VALUE`）。
    #[must_use]
    fn order(&self) -> i32 {
        i32::MAX
    }
}
