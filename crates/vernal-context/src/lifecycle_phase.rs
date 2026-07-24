//! 生命周期阶段对象。

use std::fmt;

/// 组件生命周期失败所处的阶段。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LifecyclePhase {
    /// 单例构造完成后的异步初始化。
    Initialize,
    /// 应用开始提供服务前的启动。
    Start,
    /// 回滚或正常关闭时的释放。
    Stop,
}

impl fmt::Display for LifecyclePhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Initialize => formatter.write_str("initialize"),
            Self::Start => formatter.write_str("start"),
            Self::Stop => formatter.write_str("stop"),
        }
    }
}
