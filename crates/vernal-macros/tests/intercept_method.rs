//! AOP 方法宏与 Context 内建计划目录的运行时合同测试。

use std::sync::Arc;

use vernal_aop::{CancellationToken, InvocationError, InvocationPlanCatalog, Operation};
use vernal_context::VernalApplicationBuilder;
use vernal_ioc::Component;

/// 同时验证 Component 派生接线和异步方法拦截的计算服务。
#[derive(vernal_macros::Component)]
#[component(aop)]
struct CalculatorService {
    invocation_plans: Arc<InvocationPlanCatalog>,
    cancellation: Arc<CancellationToken>,
    #[component(default)]
    bias: i32,
}

impl CalculatorService {
    /// 执行已注册操作；方法体读取 self 可验证 owned receiver 改写。
    #[vernal_macros::intercept(component = "CalculatorService")]
    async fn add(self: Arc<Self>, left: i32, right: i32) -> Result<i32, InvocationError> {
        Ok(left + right + self.bias)
    }

    /// 故意不注册计划，用于验证结构化缺失错误。
    #[vernal_macros::intercept(component = "CalculatorService")]
    async fn unplanned(self: Arc<Self>) -> Result<(), InvocationError> {
        Ok(())
    }
}

#[tokio::test]
async fn intercepted_method_uses_context_local_plan_and_cancellation() {
    let operation = Operation::new("CalculatorService", "add");
    let mut application =
        VernalApplicationBuilder::current().expect("Tokio runtime should be available");
    application.operation(operation);
    application
        .register(CalculatorService::definition())
        .expect("derived AOP component should register");
    let context = application.build().expect("application should build");
    context.refresh().await.expect("context should refresh");
    context.start().await.expect("context should start");
    let calculator = context
        .container()
        .resolve::<CalculatorService>()
        .expect("calculator should resolve");

    assert_eq!(
        Arc::clone(&calculator)
            .add(20, 22)
            .await
            .expect("intercepted call should succeed"),
        42
    );
    assert!(matches!(
        Arc::clone(&calculator).unplanned().await,
        Err(InvocationError::PlanNotFound { .. })
    ));

    context.close().await.expect("context should close");
    assert!(matches!(
        Arc::clone(&calculator).add(1, 1).await,
        Err(InvocationError::Cancelled)
    ));
}
