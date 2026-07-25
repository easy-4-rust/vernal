//! 周期任务并发与执行次数探针对象。

use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::Notify;

/// 记录任务执行次数、当前并发数和历史最大并发数。
#[derive(Default)]
pub struct ScheduledTaskProbe {
    executions: AtomicUsize,
    active: AtomicUsize,
    max_active: AtomicUsize,
    changed: Notify,
}

impl ScheduledTaskProbe {
    /// 标记一次执行开始，并更新无锁最大并发快照。
    pub fn begin(&self) {
        self.executions.fetch_add(1, Ordering::SeqCst);
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_active.fetch_max(active, Ordering::SeqCst);
        self.changed.notify_waiters();
    }

    /// 标记一次执行结束。
    pub fn end(&self) {
        self.active.fetch_sub(1, Ordering::SeqCst);
        self.changed.notify_waiters();
    }

    /// 返回累计执行次数。
    pub fn executions(&self) -> usize {
        self.executions.load(Ordering::SeqCst)
    }

    /// 返回历史最大并发数。
    pub fn max_active(&self) -> usize {
        self.max_active.load(Ordering::SeqCst)
    }

    /// 等待累计执行次数达到目标。
    pub async fn wait_for_executions(&self, expected: usize) {
        loop {
            let changed = self.changed.notified();
            if self.executions() >= expected {
                return;
            }
            changed.await;
        }
    }
}
