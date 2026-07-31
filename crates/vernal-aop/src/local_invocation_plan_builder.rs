//! 本地调用计划建造器对象。

use std::collections::HashMap;

use crate::{
    LocalAdvisor, LocalInvocationPlan, LocalInvocationPlanCatalog, Operation,
    OperationMetadataConflictError, operation_declaration_set::OperationDeclarationSet,
};

/// 收集本地顾问并为具体操作生成不可变 Local-AOP 计划。
#[derive(Default)]
pub struct LocalInvocationPlanBuilder {
    advisors: Vec<LocalAdvisor>,
}

impl LocalInvocationPlanBuilder {
    /// 创建空本地计划建造器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个本地顾问并保留稳定注册顺序。
    pub fn register(&mut self, advisor: LocalAdvisor) -> &mut Self {
        self.advisors.push(advisor);
        self
    }

    /// 为指定操作匹配并编译不可变本地调用计划。
    #[must_use]
    pub fn build(&self, operation: Operation) -> LocalInvocationPlan {
        let mut matched: Vec<&LocalAdvisor> = self
            .advisors
            .iter()
            .filter(|advisor| advisor.matches(&operation))
            .collect();

        // 稳定排序使相同 order 继续按注册顺序执行。
        matched.sort_by_key(|advisor| advisor.order());
        let interceptors = matched.into_iter().map(LocalAdvisor::interceptor).collect();
        LocalInvocationPlan::new(operation, interceptors)
    }

    /// 为一组操作批量编译不可变本地调用计划目录。
    ///
    /// 完全相同的重复声明会去重；同一稳定身份出现不同标签或限定符时拒绝整个
    /// 目录，Send 与 Local 执行平面因而共享相同的元数据确定性。
    ///
    /// # Errors
    ///
    /// 同一组件与方法出现冲突声明元数据时返回
    /// [`OperationMetadataConflictError`]。
    pub fn build_catalog(
        &self,
        operations: impl IntoIterator<Item = Operation>,
    ) -> Result<LocalInvocationPlanCatalog, OperationMetadataConflictError> {
        let declarations = OperationDeclarationSet::collect(operations)?;
        let plans = declarations
            .into_operations()
            .map(|operation| {
                let plan = self.build(operation.clone());
                (operation, plan)
            })
            .collect::<HashMap<_, _>>();
        Ok(LocalInvocationPlanCatalog::new(plans))
    }

    /// 返回已注册本地顾问数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.advisors.len()
    }

    /// 返回是否尚未注册任何本地顾问。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.advisors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_invocation_plan_builder_register() {
        let mut builder = LocalInvocationPlanBuilder::new();
        assert_eq!(builder.build(crate::Operation::new("Service", "method")).operation().component(), "Service");
    }
}
