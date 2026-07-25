//! 高层应用构建器的 Operation 元数据冲突传播合同测试。

use vernal_aop::Operation;
use vernal_context::{ApplicationBuildError, VernalApplicationBuilder};

#[tokio::test]
async fn conflicting_operation_metadata_fails_before_context_is_published() {
    let secured = Operation::new("OrderService", "create")
        .with_tag("secured")
        .expect("valid declaration tag");
    let audited = Operation::new("OrderService", "create")
        .with_tag("audited")
        .expect("valid declaration tag");
    let mut application =
        VernalApplicationBuilder::current().expect("Tokio runtime should be available");
    application.operation(secured.clone()).operation(audited);

    let Err(error) = application.build() else {
        panic!("conflicting operation metadata must reject application construction");
    };

    assert!(matches!(
        error,
        ApplicationBuildError::OperationMetadata { source }
            if source.operation() == &secured
    ));
}
