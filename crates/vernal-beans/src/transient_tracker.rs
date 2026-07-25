//! Transient 实例追踪器。
//!
//! 追踪所有通过 Container 创建的 Transient 实例的弱引用。
//! Container 关闭时，上层（如 vernal-context）可通过此追踪器通知仍存活的实例执行清理。
//!
//! # 设计来源
//!
//! 对标 tx_di 的 `Store.prototype_instances`（`Weak` 引用 + `shutdown_prototypes()`）。
//! 与 tx_di 的区别：vernal-beans 只负责追踪，不直接调用 shutdown（shutdown 由
//! vernal-context 的 `Lifecycle` trait 定义，vernal-beans 不依赖 vernal-context）。

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};

/// Transient 实例追踪器。
///
/// 追踪所有通过 Container 创建的 Transient 实例的弱引用。
/// 弱引用确保追踪器不阻止实例被正常回收。
///
/// # 使用方式
///
/// 1. Container 构造 Transient 实例后，调用 `track()` 记录弱引用
/// 2. 上层在关闭时调用 `surviving_instances()` 获取仍存活的实例
/// 3. 对存活实例执行清理逻辑（如调用 Lifecycle::stop）
pub struct TransientTracker {
    /// 按组件 TypeId 分组的弱引用集合
    instances: Mutex<HashMap<TypeId, Vec<Weak<dyn Any + Send + Sync>>>>,
}

impl TransientTracker {
    /// 创建空的追踪器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            instances: Mutex::new(HashMap::new()),
        }
    }

    /// 记录一个新创建的 Transient 实例的弱引用。
    ///
    /// # 参数
    /// - `type_id`：组件的 TypeId
    /// - `instance`：刚创建的实例的 Arc（内部转为 Weak 存储）
    pub fn track(&self, type_id: TypeId, instance: &Arc<dyn Any + Send + Sync>) {
        let weak = Arc::downgrade(instance);
        let mut instances = self
            .instances
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        instances.entry(type_id).or_default().push(weak);
    }

    /// 获取指定类型所有仍存活的 Transient 实例。
    ///
    /// 返回的 Vec 包含所有尚未被回收的 `Arc` 引用。
    /// 已被回收的实例（`Weak` 升级失败）会被自动过滤。
    ///
    /// # 参数
    /// - `type_id`：要查询的组件 TypeId
    #[must_use]
    pub fn surviving_instances(&self, type_id: TypeId) -> Vec<Arc<dyn Any + Send + Sync>> {
        // 获取锁后遍历指定类型的弱引用列表
        let mut instances = self
            .instances
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(weak_list) = instances.get_mut(&type_id) {
            // 尝试升级所有弱引用为强引用，升级失败（实例已被回收）的自动过滤
            let alive: Vec<_> = weak_list.iter().filter_map(Weak::upgrade).collect();
            // 清理已失效的弱引用，防止 Weak 指针堆积导致内存泄漏
            weak_list.retain(|w| w.strong_count() > 0);
            alive
        } else {
            // 该类型从未创建过 Transient 实例，返回空集合
            Vec::new()
        }
    }

    /// 获取所有类型的仍存活实例数量。
    ///
    /// 用于诊断和监控。
    #[must_use]
    pub fn total_surviving(&self) -> usize {
        let instances = self
            .instances
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        instances
            .values()
            .map(|weak_list| weak_list.iter().filter(|w| w.strong_count() > 0).count())
            .sum()
    }

    /// 清空所有追踪记录。
    ///
    /// 在 Container 关闭后调用，释放所有弱引用。
    pub fn clear(&self) {
        let mut instances = self
            .instances
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        instances.clear();
    }
}

impl Default for TransientTracker {
    fn default() -> Self {
        Self::new()
    }
}
