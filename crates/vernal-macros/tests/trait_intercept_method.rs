//! Trait 默认异步方法 AOP 织入运行合同测试。

mod trait_intercept_support;

use trait_intercept_support::{TraitCalculatorPort, TraitCalculatorService};
use vernal_aop::InvocationError;
use vernal_context::VernalApplicationBuilder;
use vernal_ioc::Component;

#[tokio::test]
async fn trait_default_methods_use_implementor_context_and_ufcs_descriptors() {
    let operation =
        vernal_macros::operation!(<TraitCalculatorService as TraitCalculatorPort>::trait_echo);
    assert_eq!(operation.component(), "TraitCalculatorPort");
    assert!(operation.metadata().has_tag("trait"));
    assert!(operation.metadata().has_tag("generic"));
    let implementor_operation = vernal_macros::operation!(
        <TraitCalculatorService as TraitCalculatorPort>::implementor_name
    );
    assert_eq!(
        implementor_operation.component(),
        std::any::type_name::<TraitCalculatorService>()
    );

    let mut application =
        VernalApplicationBuilder::current().expect("Tokio runtime should be available");
    application
        .operation(operation)
        .operation(implementor_operation);
    application
        .register(TraitCalculatorService::definition())
        .expect("trait service should register");
    let context = application.build().expect("application should build");
    context.refresh().await.expect("context should refresh");
    context.start().await.expect("context should start");
    let service = context
        .container()
        .resolve::<TraitCalculatorService>()
        .expect("trait service should resolve");

    assert_eq!(
        service
            .trait_echo(42_i32)
            .await
            .expect("i32 default method should succeed"),
        42
    );
    assert_eq!(
        service
            .trait_echo(String::from("vernal"))
            .await
            .expect("String default method should succeed"),
        "vernal"
    );
    assert_eq!(
        service
            .implementor_name()
            .await
            .expect("implementor identity method should succeed"),
        std::any::type_name::<TraitCalculatorService>()
    );

    context.close().await.expect("context should close");
    assert!(matches!(
        service.trait_echo(7_i32).await,
        Err(InvocationError::Cancelled)
    ));
}
