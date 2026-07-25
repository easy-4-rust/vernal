//! 应用启动顺序探针对象。

use std::sync::Mutex;

use tokio::sync::Notify;

/// 保存测试观察到的启动与回滚步骤。
#[derive(Default)]
pub struct StartupProbe {
    steps: Mutex<Vec<&'static str>>,
    changed: Notify,
}

impl StartupProbe {
    /// 追加一个稳定步骤并唤醒等待者。
    pub fn push(&self, step: &'static str) {
        self.steps
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(step);
        self.changed.notify_waiters();
    }

    /// 返回当前步骤快照。
    pub fn snapshot(&self) -> Vec<&'static str> {
        self.steps
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// 等待至少出现指定数量的步骤。
    pub async fn wait_for_count(&self, expected: usize) {
        loop {
            let changed = self.changed.notified();
            if self.snapshot().len() >= expected {
                return;
            }
            changed.await;
        }
    }
}
