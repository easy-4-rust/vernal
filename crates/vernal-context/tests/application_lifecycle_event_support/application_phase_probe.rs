//! 应用阶段事件测试探针对象。

use std::sync::Mutex;
use std::time::Duration;

use tokio::sync::Notify;

/// 记录监听器已经完成处理的应用阶段。
#[derive(Default)]
pub struct ApplicationPhaseProbe {
    phases: Mutex<Vec<&'static str>>,
    changed: Notify,
}

impl ApplicationPhaseProbe {
    /// 记录一个低基数应用阶段并唤醒等待者。
    pub fn record(&self, phase: &'static str) {
        self.phases
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(phase);
        self.changed.notify_waiters();
    }

    /// 返回当前阶段记录的拥有型快照。
    #[must_use]
    pub fn snapshot(&self) -> Vec<&'static str> {
        self.phases
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// 在有界时间内等待指定阶段被监听器处理。
    pub async fn wait_for(&self, expected: &'static str) {
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                let changed = self.changed.notified();
                if self.snapshot().contains(&expected) {
                    return;
                }
                changed.await;
            }
        })
        .await
        .expect("application lifecycle event should be handled");
    }
}
