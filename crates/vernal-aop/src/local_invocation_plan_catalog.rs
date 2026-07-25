//! 一次封存的本地调用计划目录对象。

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use crate::{InvocationPlanCatalogInitializationError, LocalInvocationPlan, Operation};

/// 保存应用上下文中全部预编译 Local-AOP 调用计划。
///
/// 目录和计划本身可以进入 `ApplicationContext` 原生组件图；只有调用时创建的
/// Future、目标、返回值与错误不跨线程移动。高层 Context 可以先把待封存目录
/// 注册进图，再由最终 Container 解析 `LocalInterceptor` 组件，封存后运行期仍是
/// 无锁只读对象。目录键只使用稳定操作身份；声明元数据冲突在构建阶段 fail-closed。
#[derive(Clone)]
pub struct LocalInvocationPlanCatalog {
    plans: Arc<OnceLock<Arc<HashMap<Operation, LocalInvocationPlan>>>>,
}

impl LocalInvocationPlanCatalog {
    /// 由本地调用计划建造器创建不可变目录。
    pub(crate) fn new(plans: HashMap<Operation, LocalInvocationPlan>) -> Self {
        Self {
            plans: Arc::new(OnceLock::from(Arc::new(plans))),
        }
    }

    /// 创建供高层 Context 完成 IoC 拦截器解析的待封存目录。
    #[doc(hidden)]
    #[must_use]
    pub fn deferred() -> Self {
        Self {
            plans: Arc::new(OnceLock::new()),
        }
    }

    /// 使用已编译本地目录一次性封存当前对象。
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

    /// 返回指定操作对应的预编译本地调用计划。
    #[must_use]
    pub fn get(&self, operation: &Operation) -> Option<&LocalInvocationPlan> {
        self.plans.get()?.get(operation)
    }

    /// 返回目录中的唯一操作数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.plans.get().map_or(0, |plans| plans.len())
    }

    /// 返回全部本地计划匹配的拦截器槽位总数。
    #[must_use]
    pub fn interceptor_count(&self) -> usize {
        self.plans.get().map_or(0, |plans| {
            plans.values().map(LocalInvocationPlan::len).sum()
        })
    }

    /// 返回目录是否不包含任何本地调用计划。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.plans.get().is_none_or(|plans| plans.is_empty())
    }
}

impl Default for LocalInvocationPlanCatalog {
    /// 创建已经封存的空本地目录。
    fn default() -> Self {
        Self::new(HashMap::new())
    }
}
