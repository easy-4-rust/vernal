//! Container 内部的组件解析追踪对象。

use std::{
    collections::HashSet,
    sync::{Mutex, MutexGuard},
};

use crate::ComponentKey;

/// 记录一个 Container 中曾经成功解析过的组件定义。
///
/// 追踪器只保存 [`ComponentKey`]，不保存实例、工厂、调用栈、请求标识或参数值。
/// 它由 Container 独占，因此两个使用同一 Registry 的应用上下文仍拥有完全隔离的
/// 使用记录；同步使用标准 Mutex，是因为 Container 的解析入口本身是同步 API。
#[derive(Default)]
pub(crate) struct ResolutionTracker {
    resolved: Mutex<HashSet<ComponentKey>>,
}

impl ResolutionTracker {
    /// 创建尚未记录任何成功解析的追踪器。
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// 在组件定义完成作用域解析后记录其稳定身份。
    ///
    /// 重复解析 Singleton、Transient 或 Custom Scope 组件只保留一个定义身份；
    /// 构造失败、Scope 未激活或候选选择失败不会调用该方法，因此不会制造假阳性。
    pub(crate) fn record(&self, key: &ComponentKey) {
        self.lock().insert(key.clone());
    }

    /// 返回调用时刻已经成功解析的定义集合快照。
    ///
    /// 克隆后立即释放锁，后续诊断排序和字符串转换不会阻塞并发组件解析。
    pub(crate) fn snapshot(&self) -> HashSet<ComponentKey> {
        self.lock().clone()
    }

    /// 获取内部集合并在 Mutex 被 panic 污染时保留已经记录的安全值。
    fn lock(&self) -> MutexGuard<'_, HashSet<ComponentKey>> {
        self.resolved
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
