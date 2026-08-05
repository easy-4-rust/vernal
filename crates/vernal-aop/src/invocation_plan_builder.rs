//! 调用计划建造器对象。

use std::collections::HashMap;

use crate::{
    Advisor, InvocationPlan, InvocationPlanCatalog, Operation, OperationMetadataConflictError,
    operation_declaration_set::OperationDeclarationSet,
};

/// 收集顾问并为具体操作生成不可变调用计划。
///
/// 切点只在计划构建阶段匹配，不进入每次业务调用的热路径。应用上下文可以在
/// refresh 阶段为所有组件方法预编译计划，之后安全地跨 Tokio task 共享。
#[derive(Default)]
pub struct InvocationPlanBuilder {
    advisors: Vec<Advisor>,
}

impl InvocationPlanBuilder {
    /// 创建空计划建造器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个顾问并保留其稳定注册顺序。
    pub fn register(&mut self, advisor: Advisor) -> &mut Self {
        self.advisors.push(advisor);
        self
    }

    /// 为指定操作匹配并编译不可变调用计划。
    #[must_use]
    pub fn build(&self, operation: Operation) -> InvocationPlan {
        let mut matched: Vec<&Advisor> = self
            .advisors
            .iter()
            .filter(|advisor| advisor.matches(&operation))
            .collect();

        // `sort_by_key` 是稳定排序，相同 order 自动保留注册顺序。
        matched.sort_by_key(|advisor| advisor.order());
        let interceptors = matched.into_iter().map(Advisor::interceptor).collect();
        InvocationPlan::new(operation, interceptors)
    }

    /// 为一组组件操作批量编译不可变调用计划目录。
    ///
    /// 完全相同的重复声明只保留一个计划；同一稳定身份出现不同标签或限定符时
    /// fail-closed，避免 `HashMap` 覆盖顺序决定最终切点集合。
    ///
    /// # Errors
    ///
    /// 同一组件与方法出现冲突声明元数据时返回
    /// [`OperationMetadataConflictError`]。
    pub fn build_catalog(
        &self,
        operations: impl IntoIterator<Item = Operation>,
    ) -> Result<InvocationPlanCatalog, OperationMetadataConflictError> {
        let declarations = OperationDeclarationSet::collect(operations)?;
        let plans = declarations
            .into_operations()
            .map(|operation| {
                let plan = self.build(operation.clone());
                (operation, plan)
            })
            .collect::<HashMap<_, _>>();
        Ok(InvocationPlanCatalog::new(plans))
    }

    /// 返回已注册顾问数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.advisors.len()
    }

    /// 返回是否尚未注册任何顾问。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.advisors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AnyPointcut, Interceptor, Invocation, Next, Operation};

    struct TestInterceptor;
    impl Interceptor for TestInterceptor {
        fn intercept<'a>(
            &'a self,
            invocation: std::sync::Arc<Invocation>,
            next: Next<'a>,
        ) -> crate::InvocationFuture<'a> {
            next.run(invocation)
        }
    }

    #[test]
    fn builder_new_is_empty() {
        let builder = InvocationPlanBuilder::new();
        assert_eq!(builder.len(), 0);
        assert!(builder.is_empty());
    }

    #[test]
    fn builder_register_increases_len() {
        let mut builder = InvocationPlanBuilder::new();
        let advisor = Advisor::new(AnyPointcut::new(), TestInterceptor, 0);
        builder.register(advisor);
        assert_eq!(builder.len(), 1);
        assert!(!builder.is_empty());
    }

    #[test]
    fn builder_build_empty_returns_empty_plan() {
        let builder = InvocationPlanBuilder::new();
        let op = Operation::new("Service", "method");
        let plan = builder.build(op.clone());
        assert_eq!(plan.len(), 0);
        assert!(plan.is_empty());
        assert_eq!(plan.operation().component(), "Service");
    }

    #[test]
    fn builder_build_matching_advisor() {
        let mut builder = InvocationPlanBuilder::new();
        let advisor = Advisor::new(AnyPointcut::new(), TestInterceptor, 0);
        builder.register(advisor);
        let op = Operation::new("Service", "method");
        let plan = builder.build(op);
        assert_eq!(plan.len(), 1);
    }

    #[test]
    fn builder_build_respects_order() {
        let mut builder = InvocationPlanBuilder::new();
        builder.register(Advisor::new(AnyPointcut::new(), TestInterceptor, 2));
        builder.register(Advisor::new(AnyPointcut::new(), TestInterceptor, 1));
        let op = Operation::new("Service", "method");
        let plan = builder.build(op);
        assert_eq!(plan.len(), 2);
    }

    #[test]
    fn builder_build_catalog() {
        let mut builder = InvocationPlanBuilder::new();
        builder.register(Advisor::new(AnyPointcut::new(), TestInterceptor, 0));
        let ops = vec![
            Operation::new("Service", "method1"),
            Operation::new("Service", "method2"),
        ];
        let catalog = builder.build_catalog(ops);
        assert!(catalog.is_ok());
    }
}
