//! AOP 方法宏与 Context 内建计划目录的运行时合同测试。

mod intercept_support;

use std::sync::Arc;

use intercept_support::{MetadataProbeInterceptor, StatefulService};
use vernal_aop::{
    Advisor, CancellationToken, InvocationError, InvocationPlanCatalog, PointcutExt,
    QualifierPointcut, TagPointcut,
};
use vernal_context::VernalApplicationBuilder;
use vernal_beans::Component;

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
    /// 执行带静态标签和限定符的已注册操作。
    #[vernal_macros::intercept(
        component = "CalculatorService",
        tags = ["secured", "calculation", "secured"],
        qualifier = "primary"
    )]
    async fn add(self: Arc<Self>, left: i32, right: i32) -> Result<i32, InvocationError> {
        Ok(left + right + self.bias)
    }

    /// 使用普通共享借用接收器，并让引用参数安全跨越一次 `.await`。
    #[vernal_macros::intercept(
        component = "CalculatorService",
        tags = ["secured", "borrowed"],
        qualifier = "secondary"
    )]
    async fn borrowed_add(&self, left: &i32, right: i32) -> Result<i32, InvocationError> {
        tokio::task::yield_now().await;
        Ok(*left + right + self.bias)
    }

    /// 验证 owned Arc 目标也可以携带满足擦除边界的泛型参数。
    #[vernal_macros::intercept(component = "CalculatorService", tags = ["generic", "owned"])]
    async fn owned_echo<T>(self: Arc<Self>, value: T) -> Result<T, InvocationError>
    where
        T: Send + Sync + 'static,
    {
        tokio::task::yield_now().await;
        Ok(value)
    }

    /// 故意不注册计划，用于验证结构化缺失错误。
    #[vernal_macros::intercept]
    async fn unplanned(self: Arc<Self>) -> Result<(), InvocationError> {
        Ok(())
    }
}

#[tokio::test]
async fn intercepted_method_uses_context_local_plan_and_cancellation() {
    let operation = vernal_macros::operation!(CalculatorService::add);
    let borrowed_operation = vernal_macros::operation!(CalculatorService::borrowed_add);
    let owned_generic_operation = vernal_macros::operation!(CalculatorService::owned_echo);
    assert_eq!(
        operation
            .metadata()
            .tags()
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>(),
        ["calculation", "secured"]
    );
    assert_eq!(operation.metadata().qualifier(), Some("primary"));
    assert_eq!(borrowed_operation.metadata().qualifier(), Some("secondary"));
    let unplanned_operation = vernal_macros::operation!(CalculatorService::unplanned);
    assert_eq!(
        unplanned_operation.component(),
        std::any::type_name::<CalculatorService>()
    );

    let probe = MetadataProbeInterceptor::new();
    let secured_primary = TagPointcut::new("secured")
        .expect("static tag should be valid")
        .and(QualifierPointcut::new("primary").expect("static qualifier should be valid"));
    let mut application =
        VernalApplicationBuilder::current().expect("Tokio runtime should be available");
    application
        .operation(operation)
        .operation(borrowed_operation)
        .operation(owned_generic_operation)
        .advisor(Advisor::new(secured_primary, probe.clone(), 0));
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
    let observations = probe.snapshot();
    assert_eq!(observations.len(), 1);
    assert!(observations[0].has_tag("secured"));
    assert_eq!(observations[0].qualifier(), Some("primary"));
    let borrowed_left = 40;
    assert_eq!(
        calculator
            .borrowed_add(&borrowed_left, 2)
            .await
            .expect("borrowed intercepted call should succeed"),
        42
    );
    assert_eq!(
        Arc::clone(&calculator)
            .owned_echo(String::from("owned"))
            .await
            .expect("owned generic call should succeed"),
        "owned"
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
    assert!(matches!(
        calculator.borrowed_add(&1, 1).await,
        Err(InvocationError::Cancelled)
    ));
}

#[tokio::test]
async fn intercepted_method_supports_mutable_and_generic_implementation_signatures() {
    let mut application =
        VernalApplicationBuilder::current().expect("Tokio runtime should be available");
    application
        .operation(vernal_macros::operation!(StatefulService::replace))
        .operation(vernal_macros::operation!(StatefulService::echo))
        .operation(vernal_macros::operation!(StatefulService::clone_borrowed))
        .operation(vernal_macros::operation!(StatefulService::echo_array));
    application
        .register(StatefulService::definition())
        .expect("stateful AOP component should register");
    let context = application.build().expect("application should build");
    context.refresh().await.expect("context should refresh");
    context.start().await.expect("context should start");

    // Transient 组件不会被 Container 缓存，因此解析得到的 Arc 仍可唯一借用。
    // 可变业务 Future、接收器和参数都被限制在本次 invoke_borrowed().await 内。
    let mut stateful = context
        .container()
        .resolve::<StatefulService>()
        .expect("stateful service should resolve");
    let stateful_mut =
        Arc::get_mut(&mut stateful).expect("transient component Arc should have one owner");
    assert_eq!(
        stateful_mut
            .replace(42)
            .await
            .expect("mutable intercepted call should succeed"),
        0
    );
    assert_eq!(stateful_mut.value(), 42);

    // 两个 type 泛型单态化调用共享稳定 Operation 身份，并按每次实际 T 恢复返回值。
    assert_eq!(
        stateful_mut
            .echo(42_i32)
            .await
            .expect("generic i32 call should succeed"),
        42
    );
    assert_eq!(
        stateful_mut
            .echo(String::from("vernal"))
            .await
            .expect("generic String call should succeed"),
        "vernal"
    );
    let borrowed = String::from("borrowed");
    assert_eq!(
        stateful_mut
            .clone_borrowed(&borrowed)
            .await
            .expect("lifetime generic call should succeed"),
        "borrowed"
    );
    assert_eq!(
        stateful_mut
            .echo_array([1_u8, 2, 3, 4])
            .await
            .expect("const generic call should succeed"),
        [1, 2, 3, 4]
    );

    context.close().await.expect("context should close");
    assert!(matches!(
        stateful_mut.replace(7).await,
        Err(InvocationError::Cancelled)
    ));
}
