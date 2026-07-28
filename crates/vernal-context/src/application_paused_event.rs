//! 应用 Context 进入暂停状态事件值对象。

/// 表示当前 [`crate::ApplicationContext`] 已经进入 `Paused` 状态。
///
/// 对标 Spring 7.0 `org.springframework.context.event.ContextPausedEvent`：
/// 由启动协调器在 `LifecycleProcessor.onPause()` 完成后、即所有声明
/// `is_pauseable() == true` 的 Lifecycle 组件按依赖逆序 `pause()` 成功后发布。
/// Context-local 强类型监听器可以通过普通 `IoC` 订阅响应该事件，例如冻结
/// 消费者游标、停止接收新请求或对缓存做快照。
///
/// 事件只表示状态事实，不是同步屏障：`pause()` 不等待监听器处理完成；处理错误
/// 仍会作为受管任务失败取消应用，并通过 [`crate::ApplicationContext::close`]
/// 路径返回结构化错误。
///
/// 与 `ContextRefreshedEvent` / `ContextStartedEvent` 等其他上下文事件一致，本
/// 事件不携带 Context 指针或运行期数据，避免形成所有权环。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ApplicationPausedEvent;

impl ApplicationPausedEvent {
    /// 创建不携带运行时数据的暂停完成事件。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}
