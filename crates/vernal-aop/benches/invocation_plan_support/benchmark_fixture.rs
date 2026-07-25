//! AOP 调用计划基准夹具对象。

use std::{hint::black_box, sync::Arc};

use vernal_aop::{
    Advisor, AnyPointcut, Invocation, InvocationPlan, InvocationPlanBuilder, InvocationTarget,
    InvocationValue, Operation,
};

use super::PassthroughInterceptor;

/// 持有可跨采样复用的调用、目标与三种预编译计划。
///
/// 夹具在计时区间外构建，模拟 `ApplicationContext` 已完成 refresh、业务请求只执行
/// 不可变计划的生产状态。每次目标调用仍会创建真实的类型擦除返回值。
pub struct BenchmarkFixture {
    invocation: Arc<Invocation>,
    target: Arc<InvocationTarget>,
    empty_plan: InvocationPlan,
    one_plan: InvocationPlan,
    four_plan: InvocationPlan,
}

impl BenchmarkFixture {
    /// 创建共享调用对象、共享目标以及零、一、四拦截器计划。
    #[must_use]
    pub fn new() -> Self {
        let operation = Operation::new("BenchmarkService", "execute");
        let invocation = Invocation::new(operation.clone()).shared();
        let target: Arc<InvocationTarget> =
            Arc::new(|_invocation| Box::pin(async { Ok(Box::new(42_u64) as InvocationValue) }));

        let empty_plan = InvocationPlanBuilder::new().build(operation.clone());

        let mut one_builder = InvocationPlanBuilder::new();
        one_builder.register(Advisor::new(AnyPointcut::new(), PassthroughInterceptor, 0));
        let one_plan = one_builder.build(operation.clone());

        let mut four_builder = InvocationPlanBuilder::new();
        for order in 0..4 {
            four_builder.register(Advisor::new(
                AnyPointcut::new(),
                PassthroughInterceptor,
                order,
            ));
        }
        let four_plan = four_builder.build(operation);

        Self {
            invocation,
            target,
            empty_plan,
            one_plan,
            four_plan,
        }
    }

    /// 执行不含拦截器的真实调用计划。
    pub async fn invoke_empty(&self) {
        black_box(self.invoke(&self.empty_plan).await);
    }

    /// 执行包含一个透传拦截器的真实调用计划。
    pub async fn invoke_one(&self) {
        black_box(self.invoke(&self.one_plan).await);
    }

    /// 执行包含四个透传拦截器的真实调用计划。
    pub async fn invoke_four(&self) {
        black_box(self.invoke(&self.four_plan).await);
    }

    /// 执行指定计划并恢复业务返回值的静态类型。
    async fn invoke(&self, plan: &InvocationPlan) -> u64 {
        let value = plan
            .invoke(Arc::clone(&self.invocation), Arc::clone(&self.target))
            .await
            .expect("benchmark invocation should succeed");
        *value
            .downcast::<u64>()
            .expect("benchmark target should return u64")
    }
}

impl Default for BenchmarkFixture {
    fn default() -> Self {
        Self::new()
    }
}
