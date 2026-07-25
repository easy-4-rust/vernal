//! 应用模块测试观测器对象。

use std::sync::{Mutex, MutexGuard};

/// 以线程安全顺序列表保存模块生命周期和拦截器执行证据。
#[derive(Default)]
pub struct ModuleProbe {
    events: Mutex<Vec<&'static str>>,
}

impl ModuleProbe {
    /// 追加一个静态阶段名称。
    pub fn push(&self, event: &'static str) {
        self.lock().push(event);
    }

    /// 返回当前事件快照。
    #[must_use]
    pub fn snapshot(&self) -> Vec<&'static str> {
        self.lock().clone()
    }

    /// 取得内部列表，并在测试 panic 污染锁时保留已产生的证据。
    fn lock(&self) -> MutexGuard<'_, Vec<&'static str>> {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
