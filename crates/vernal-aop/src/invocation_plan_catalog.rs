//! 一次封存的调用计划目录对象。

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use crate::{InvocationPlan, InvocationPlanCatalogInitializationError, Operation};

/// 保存应用上下文中全部预编译 AOP 调用计划。
///
/// 目录通常由 [`crate::InvocationPlanBuilder`] 直接构建。高层 Context 还可以先
/// 创建待封存目录，把它作为普通 Rust 对象注册进 `IoC` 图，再从同一个 Container
/// 解析由组件实现的拦截器并一次性封存。封存后没有修改 API，运行期仍只执行无锁
/// 读取，不使用进程级全局表或实例指针映射。
///
/// 相同 [`Operation`] 的重复声明会被合并为一个计划，因为它们基于同一组 Advisor
/// 编译，结果完全等价。
#[derive(Clone)]
pub struct InvocationPlanCatalog {
    plans: Arc<OnceLock<Arc<HashMap<Operation, InvocationPlan>>>>,
}

impl InvocationPlanCatalog {
    /// 由调用计划建造器创建不可变目录。
    pub(crate) fn new(plans: HashMap<Operation, InvocationPlan>) -> Self {
        Self {
            plans: Arc::new(OnceLock::from(Arc::new(plans))),
        }
    }

    /// 创建尚未写入计划的待封存目录。
    ///
    /// 该入口服务于需要先把目录放入依赖图、再由同一个 Container 构造拦截器的
    /// 框架装配路径。调用方必须在公开应用对象之前调用 [`Self::initialize_from`]；
    /// 未封存目录的只读方法安全地表现为空目录。
    #[doc(hidden)]
    #[must_use]
    pub fn deferred() -> Self {
        Self {
            plans: Arc::new(OnceLock::new()),
        }
    }

    /// 使用已编译目录一次性封存当前待初始化目录。
    ///
    /// 克隆的是底层不可变 `Arc<HashMap<...>>`，不会复制全部计划。成功后所有既有
    /// Clone 立即观察到同一份内容；重复调用返回结构化错误并保留第一次结果。
    ///
    /// # Errors
    ///
    /// 当前目录已经封存时返回
    /// [`InvocationPlanCatalogInitializationError`]。
    #[doc(hidden)]
    pub fn initialize_from(
        &self,
        compiled: &Self,
    ) -> Result<(), InvocationPlanCatalogInitializationError> {
        let plans = compiled.plans.get().cloned().unwrap_or_default();
        self.plans
            .set(plans)
            .map_err(|_| InvocationPlanCatalogInitializationError)
    }

    /// 返回指定组件操作对应的预编译调用计划。
    #[must_use]
    pub fn get(&self, operation: &Operation) -> Option<&InvocationPlan> {
        self.plans.get()?.get(operation)
    }

    /// 返回目录中的唯一操作数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.plans.get().map_or(0, |plans| plans.len())
    }

    /// 返回全部调用计划匹配的拦截器槽位总数。
    ///
    /// 同一个拦截器实例可能匹配多个操作，因此该值描述计划中的执行槽位，而不是
    /// 唯一拦截器对象数量。诊断报告使用它反映实际织入规模。
    #[must_use]
    pub fn interceptor_count(&self) -> usize {
        self.plans
            .get()
            .map_or(0, |plans| plans.values().map(InvocationPlan::len).sum())
    }

    /// 返回目录是否不包含任何调用计划。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.plans.get().is_none_or(|plans| plans.is_empty())
    }
}

impl Default for InvocationPlanCatalog {
    /// 创建已经封存的空目录。
    ///
    /// 普通调用方取得的默认值仍是完整有效对象；只有框架装配路径显式使用
    /// [`Self::deferred`] 才会产生短暂的待封存状态。
    fn default() -> Self {
        Self::new(HashMap::new())
    }
}
