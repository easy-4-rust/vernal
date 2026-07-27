//! 生命周期阶段对象。

use std::fmt;

/// 组件生命周期失败所处的阶段。
///
/// 与 Spring 7.0 的生命周期阶段对应：除 `Initialize / Start / Stop` 外，
/// `Pause` 用于诊断 `ApplicationContext::pause()` 期间的暂停钩子失败。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LifecyclePhase {
    /// 单例构造完成后的异步初始化。
    Initialize,
    /// 应用开始提供服务前的启动。
    Start,
    /// 回滚或正常关闭时的释放。
    Stop,
    /// Context 进入 `Paused` 状态前的暂停（对标 Spring 7.0 `onPause()`）。
    Pause,
}

impl fmt::Display for LifecyclePhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Initialize => formatter.write_str("initialize"),
            Self::Start => formatter.write_str("start"),
            Self::Stop => formatter.write_str("stop"),
            Self::Pause => formatter.write_str("pause"),
        }
    }
}
