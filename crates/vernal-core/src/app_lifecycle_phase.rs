//! 应用生命周期阶段枚举。
//!
//! 对标 Spring Framework 7 的 `Lifecycle.Phase` 与 `SmartLifecycle.getPhase()` 的 8 状态语义。
//! 定义应用层（IoC 容器 / ApplicationContext）生命周期的各个阶段，用于诊断和状态追踪。
//!
//! # 设计来源
//!
//! - Spring `Lifecycle.start()` / `stop()` 阶段
//! - Spring `SmartLifecycle.getPhase(): Integer` 的 phase 概念
//! - `tx_di` `Component::init/async_init/async_run/shutdown` 的 4 阶段扩成 8 态
//!
//! # 命名
//!
//! 之所以命名为 `AppLifecyclePhase` 而非 `LifecyclePhase`,是为了避免与
//! `vernal_context::LifecyclePhase`(3 变体: `Initialize / Start / Stop`)同名冲突。
//! 8 变体版本对标 Spring 的完整应用层阶段;3 变体版本对标 Spring `LifecycleProcessor`
//! 协调的精简阶段。

/// 应用生命周期阶段。
///
/// 用于追踪应用层（IoC 容器 / ApplicationContext）在生命周期中的当前位置，
/// 以及在关闭时确定逆序执行的起点。
///
/// # 状态机
///
/// ```text
/// Created
///   ↓
/// Initializing
///   ↓
/// Initialized
///   ↓
/// Starting
///   ↓
/// Started ─────────┐
///   ↓              │
/// Stopping ←───────┘ (关闭时逆序)
///   ↓
/// Stopped (终态)
///   or
/// Failed (任何阶段失败)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppLifecyclePhase {
    /// 尚未初始化（容器刚创建）
    Created,
    /// 正在初始化（`inner_init` 或 `initialize` 执行中）
    Initializing,
    /// 已初始化（`inner_init` 和 `initialize` 完成）
    Initialized,
    /// 正在启动（`start` 或 `async_run` 执行中）
    Starting,
    /// 已启动（所有启动钩子完成,应用正在运行）
    Started,
    /// 正在停止（`stop` 或 `shutdown` 执行中）
    Stopping,
    /// 已停止（所有停止钩子完成）
    Stopped,
    /// 启动失败（某个钩子返回错误或 panic）
    Failed,
}

impl AppLifecyclePhase {
    /// 是否处于活跃状态（`Initialized` 或 `Started`）。
    ///
    /// 对标 Spring `Lifecycle.isRunning()`。
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Initialized | Self::Started)
    }

    /// 是否处于终止状态（`Stopped` 或 `Failed`）。
    ///
    /// 终止状态下容器不再接收新请求。
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Stopped | Self::Failed)
    }

    /// 是否处于过渡状态（`Initializing` / `Starting` / `Stopping`）。
    ///
    /// 过渡状态下不应再次触发生命周期钩子。
    #[must_use]
    pub fn is_transitioning(&self) -> bool {
        matches!(self, Self::Initializing | Self::Starting | Self::Stopping)
    }

    /// 是否为失败状态。
    #[must_use]
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed)
    }

    /// 获取 Spring `SmartLifecycle.getPhase()` 等价值。
    ///
    /// 返回值约定：
    /// - `Created / Initializing → Integer.MIN_VALUE`
    /// - `Initialized → 0`
    /// - `Starting / Started → Integer.MAX_VALUE`
    /// - `Stopping / Stopped → Integer.MAX_VALUE - 1`
    /// - `Failed → Integer.MIN_VALUE + 1`
    #[must_use]
    pub fn spring_phase(&self) -> i32 {
        match self {
            Self::Created | Self::Initializing => i32::MIN,
            Self::Initialized => 0,
            Self::Starting | Self::Started => i32::MAX,
            Self::Stopping | Self::Stopped => i32::MAX - 1,
            Self::Failed => i32::MIN + 1,
        }
    }
}

impl std::fmt::Display for AppLifecyclePhase {
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

// ─── 单元测试 ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_active_returns_true_for_initialized_and_started() {
        assert!(AppLifecyclePhase::Initialized.is_active());
        assert!(AppLifecyclePhase::Started.is_active());
    }

    #[test]
    fn is_active_returns_false_for_other_phases() {
        assert!(!AppLifecyclePhase::Created.is_active());
        assert!(!AppLifecyclePhase::Initializing.is_active());
        assert!(!AppLifecyclePhase::Starting.is_active());
        assert!(!AppLifecyclePhase::Stopping.is_active());
        assert!(!AppLifecyclePhase::Stopped.is_active());
        assert!(!AppLifecyclePhase::Failed.is_active());
    }

    #[test]
    fn is_terminal_returns_true_for_stopped_and_failed() {
        assert!(AppLifecyclePhase::Stopped.is_terminal());
        assert!(AppLifecyclePhase::Failed.is_terminal());
    }

    #[test]
    fn is_terminal_returns_false_for_other_phases() {
        assert!(!AppLifecyclePhase::Created.is_terminal());
        assert!(!AppLifecyclePhase::Initializing.is_terminal());
        assert!(!AppLifecyclePhase::Initialized.is_terminal());
        assert!(!AppLifecyclePhase::Starting.is_terminal());
        assert!(!AppLifecyclePhase::Started.is_terminal());
        assert!(!AppLifecyclePhase::Stopping.is_terminal());
    }

    #[test]
    fn is_transitioning_returns_true_for_three_transitional_phases() {
        assert!(AppLifecyclePhase::Initializing.is_transitioning());
        assert!(AppLifecyclePhase::Starting.is_transitioning());
        assert!(AppLifecyclePhase::Stopping.is_transitioning());
    }

    #[test]
    fn is_transitioning_returns_false_for_stable_phases() {
        assert!(!AppLifecyclePhase::Created.is_transitioning());
        assert!(!AppLifecyclePhase::Initialized.is_transitioning());
        assert!(!AppLifecyclePhase::Started.is_transitioning());
        assert!(!AppLifecyclePhase::Stopped.is_transitioning());
        assert!(!AppLifecyclePhase::Failed.is_transitioning());
    }

    #[test]
    fn is_failed_only_for_failed_phase() {
        assert!(AppLifecyclePhase::Failed.is_failed());
        for phase in [
            AppLifecyclePhase::Created,
            AppLifecyclePhase::Initializing,
            AppLifecyclePhase::Initialized,
            AppLifecyclePhase::Starting,
            AppLifecyclePhase::Started,
            AppLifecyclePhase::Stopping,
            AppLifecyclePhase::Stopped,
        ] {
            assert!(!phase.is_failed(), "{phase:?} should not be failed");
        }
    }

    #[test]
    fn spring_phase_maps_smart_lifecycle_correctly() {
        assert_eq!(AppLifecyclePhase::Created.spring_phase(), i32::MIN);
        assert_eq!(AppLifecyclePhase::Initializing.spring_phase(), i32::MIN);
        assert_eq!(AppLifecyclePhase::Initialized.spring_phase(), 0);
        assert_eq!(AppLifecyclePhase::Starting.spring_phase(), i32::MAX);
        assert_eq!(AppLifecyclePhase::Started.spring_phase(), i32::MAX);
        assert_eq!(AppLifecyclePhase::Stopping.spring_phase(), i32::MAX - 1);
        assert_eq!(AppLifecyclePhase::Stopped.spring_phase(), i32::MAX - 1);
        assert_eq!(AppLifecyclePhase::Failed.spring_phase(), i32::MIN + 1);
    }

    #[test]
    fn display_outputs_lowercase_names() {
        assert_eq!(AppLifecyclePhase::Created.to_string(), "created");
        assert_eq!(AppLifecyclePhase::Initializing.to_string(), "initializing");
        assert_eq!(AppLifecyclePhase::Initialized.to_string(), "initialized");
        assert_eq!(AppLifecyclePhase::Starting.to_string(), "starting");
        assert_eq!(AppLifecyclePhase::Started.to_string(), "started");
        assert_eq!(AppLifecyclePhase::Stopping.to_string(), "stopping");
        assert_eq!(AppLifecyclePhase::Stopped.to_string(), "stopped");
        assert_eq!(AppLifecyclePhase::Failed.to_string(), "failed");
    }

    #[test]
    fn all_8_phases_are_distinct() {
        let all = [
            AppLifecyclePhase::Created,
            AppLifecyclePhase::Initializing,
            AppLifecyclePhase::Initialized,
            AppLifecyclePhase::Starting,
            AppLifecyclePhase::Started,
            AppLifecyclePhase::Stopping,
            AppLifecyclePhase::Stopped,
            AppLifecyclePhase::Failed,
        ];
        for i in 0..all.len() {
            for j in 0..all.len() {
                if i != j {
                    assert_ne!(all[i], all[j], "phases {i} and {j} should differ");
                }
            }
        }
        assert_eq!(all.len(), 8);
    }

    #[test]
    fn phase_can_be_hashed_and_used_as_map_key() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(AppLifecyclePhase::Started);
        set.insert(AppLifecyclePhase::Stopped);
        set.insert(AppLifecyclePhase::Started); // 重复插入
        assert_eq!(set.len(), 2);
        assert!(set.contains(&AppLifecyclePhase::Started));
        assert!(!set.contains(&AppLifecyclePhase::Created));
    }
}
