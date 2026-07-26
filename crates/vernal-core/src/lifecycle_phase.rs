//! 生命周期阶段枚举。
//!
//! 定义组件生命周期的各个阶段，用于诊断和状态追踪。
//! 对标 Spring 的 `Lifecycle.Phase`。

/// 组件生命周期阶段。
///
/// 用于追踪组件在生命周期中的当前位置，
/// 以及在关闭时确定逆序执行的起点。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifecyclePhase {
    /// 尚未初始化（刚构造完成）
    Created,
    /// 正在初始化（`inner_init` 或 `initialize` 执行中）
    Initializing,
    /// 已初始化（`inner_init` 和 `initialize` 完成）
    Initialized,
    /// 正在启动（`start` 或 `async_run` 执行中）
    Starting,
    /// 已启动（所有启动钩子完成）
    Started,
    /// 正在停止（`stop` 或 `shutdown` 执行中）
    Stopping,
    /// 已停止（所有停止钩子完成）
    Stopped,
    /// 启动失败（某个钩子返回错误）
    Failed,
}

impl LifecyclePhase {
    /// 是否处于活跃状态（已初始化或已启动）。
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Initialized | Self::Started)
    }

    /// 是否处于终止状态（已停止或失败）。
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Stopped | Self::Failed)
    }

    /// 是否处于过渡状态（正在初始化、启动或停止）。
    #[must_use]
    pub fn is_transitioning(&self) -> bool {
        matches!(self, Self::Initializing | Self::Starting | Self::Stopping)
    }
}

impl std::fmt::Display for LifecyclePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Initializing => write!(f, "initializing"),
            Self::Initialized => write!(f, "initialized"),
            Self::Starting => write!(f, "starting"),
            Self::Started => write!(f, "started"),
            Self::Stopping => write!(f, "stopping"),
            Self::Stopped => write!(f, "stopped"),
            Self::Failed => write!(f, "failed"),
        }
    }
}
