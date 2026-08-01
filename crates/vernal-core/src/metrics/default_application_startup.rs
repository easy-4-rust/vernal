//! 默认应用启动。
//!
//! 对标 Spring `org.springframework.core.metrics.DefaultApplicationStartup`。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use super::application_startup::ApplicationStartup;
use super::startup_step::{SimpleStartupStep, StartupStep};

/// 默认应用启动。
///
/// 对应 Java: org.springframework.core.metrics.DefaultApplicationStartup
///
/// Spring 语义：默认启动观测器——使用原子 ID 计数器创建步骤，不进行外部
/// 记录（对标 Spring 的轻量默认实现）。
pub struct DefaultApplicationStartup {
    next_id: AtomicU64,
    current_step: Mutex<Option<u64>>,
}

impl DefaultApplicationStartup {
    /// 创建默认启动观测器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            current_step: Mutex::new(None),
        }
    }
}

impl Default for DefaultApplicationStartup {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationStartup for DefaultApplicationStartup {
    fn start(&self, name: &str) -> Box<dyn StartupStep> {
        let parent = self.current_step.lock().unwrap().unwrap_or(0);
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        Box::new(SimpleStartupStep::new(id, parent, name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_steps_with_increasing_ids() {
        // A 类（合同对齐）：对标 Spring 步骤 ID 单调递增
        let startup = DefaultApplicationStartup::new();
        let first = startup.start("phase-a");
        let second = startup.start("phase-b");
        assert_eq!(first.id(), 1);
        assert_eq!(second.id(), 2);
        assert_eq!(first.name(), "phase-a");
        assert_eq!(second.name(), "phase-b");
        assert_eq!(second.parent_id(), 0);
    }

    #[test]
    fn steps_are_independent() {
        // D 类（生命周期）：步骤可独立结束
        let startup = DefaultApplicationStartup::default();
        let mut step = startup.start("refresh");
        assert!(!step.is_ended());
        step.end();
        assert!(step.is_ended());
    }
}
