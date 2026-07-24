//! 应用关闭信号值对象。

/// 标识触发 Vernal 应用关闭的操作系统信号。
///
/// 该值会在 Context 取消应用令牌之前发布到同一 Context 的 [`crate::EventBus`]，
/// 订阅方可以记录低基数关闭原因，但不能阻止或延迟关闭。不同平台只会产生自身
/// 支持的枚举值；枚举保持完整是为了让跨平台应用使用同一事件类型。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ApplicationShutdownSignal {
    /// 终端中断，通常由 Ctrl-C 触发。
    Interrupt,
    /// Unix SIGTERM。
    Terminate,
    /// Unix SIGHUP。
    Hangup,
    /// Windows Ctrl-Break。
    Break,
    /// Windows控制台关闭事件。
    Close,
    /// Windows 系统关机事件。
    Shutdown,
}
