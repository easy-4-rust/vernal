//! 不可变本地调用计划目录对象。

use std::{collections::HashMap, sync::Arc};

use crate::{LocalInvocationPlan, Operation};

/// 保存应用上下文中全部预编译 Local-AOP 调用计划。
///
/// 目录和计划本身可以进入 `ApplicationContext` 原生组件图；只有调用时创建的
/// Future、目标、返回值与错误不跨线程移动。
#[derive(Clone, Default)]
pub struct LocalInvocationPlanCatalog {
    plans: Arc<HashMap<Operation, LocalInvocationPlan>>,
}

impl LocalInvocationPlanCatalog {
    /// 由本地调用计划建造器创建不可变目录。
    pub(crate) fn new(plans: HashMap<Operation, LocalInvocationPlan>) -> Self {
        Self {
            plans: Arc::new(plans),
        }
    }

    /// 返回指定操作对应的预编译本地调用计划。
    #[must_use]
    pub fn get(&self, operation: &Operation) -> Option<&LocalInvocationPlan> {
        self.plans.get(operation)
    }

    /// 返回目录中的唯一操作数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.plans.len()
    }

    /// 返回全部本地计划匹配的拦截器槽位总数。
    #[must_use]
    pub fn interceptor_count(&self) -> usize {
        self.plans.values().map(LocalInvocationPlan::len).sum()
    }

    /// 返回目录是否不包含任何本地调用计划。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.plans.is_empty()
    }
}
