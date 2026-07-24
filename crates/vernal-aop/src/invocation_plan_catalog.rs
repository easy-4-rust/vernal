//! 不可变调用计划目录对象。

use std::{collections::HashMap, sync::Arc};

use crate::{InvocationPlan, Operation};

/// 保存应用上下文中全部预编译 AOP 调用计划。
///
/// 目录在应用构建阶段一次性生成，运行期只执行只读查找，因此可以安全地在
/// Tokio task、Web 请求和后台任务之间共享。相同 [`Operation`] 的重复声明会
/// 被合并为一个计划，因为它们基于同一组 Advisor 编译，结果完全等价。
#[derive(Clone, Default)]
pub struct InvocationPlanCatalog {
    plans: Arc<HashMap<Operation, InvocationPlan>>,
}

impl InvocationPlanCatalog {
    /// 由调用计划建造器创建不可变目录。
    pub(crate) fn new(plans: HashMap<Operation, InvocationPlan>) -> Self {
        Self {
            plans: Arc::new(plans),
        }
    }

    /// 返回指定组件操作对应的预编译调用计划。
    #[must_use]
    pub fn get(&self, operation: &Operation) -> Option<&InvocationPlan> {
        self.plans.get(operation)
    }

    /// 返回目录中的唯一操作数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.plans.len()
    }

    /// 返回目录是否不包含任何调用计划。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.plans.is_empty()
    }
}
