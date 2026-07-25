//! 应用 Context 进入就绪状态事件值对象。

/// 表示当前 [`crate::ApplicationContext`] 已经进入 `Ready` 状态。
///
/// 启动协调器仅在全部 Lifecycle `start()`、一次性
/// [`crate::ApplicationRunner`] 成功、周期任务被监督器接受且应用尚未取消时提交
/// `Ready`，随后向 Context-local [`crate::EventBus`] 发布本事件。持续运行的
/// Ddd4r 投影、Sa-Token-Rust 安全监听或 Hutool-Rust 运维任务可以通过普通
/// `IoC` 监听/周期组件响应，不需要 tx-di 式全局 App 回调或遗弃 Tokio task。
///
/// 事件只表示状态事实，不是同步屏障。`start()` 不等待监听器处理完成；处理错误
/// 或 broadcast lag 仍会作为受管任务失败取消应用。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ApplicationReadyEvent;

impl ApplicationReadyEvent {
    /// 创建不携带运行时数据的应用就绪事件。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}
