//! Vernal Operation 身份、声明元数据和计划投影合同测试。

use std::{collections::HashSet, rc::Rc, sync::Arc};

use vernal_aop::{
    Invocation, InvocationPlanBuilder, InvocationTarget, InvocationValue,
    LocalInvocationPlanBuilder, LocalInvocationTarget, LocalInvocationValue, Operation,
    OperationMetadata, OperationMetadataError, Pointcut, PointcutExt, QualifierPointcut,
    TagPointcut,
};

#[test]
fn metadata_validates_sorts_and_deduplicates_developer_declarations() {
    let metadata = OperationMetadata::empty()
        .with_tag("secured")
        .expect("valid tag")
        .with_tag("audited")
        .expect("valid tag")
        .with_tag("secured")
        .expect("duplicate tag should coalesce")
        .with_qualifier("write")
        .expect("valid qualifier");

    assert_eq!(
        metadata
            .tags()
            .iter()
            .map(AsRef::<str>::as_ref)
            .collect::<Vec<_>>(),
        ["audited", "secured"]
    );
    assert_eq!(metadata.qualifier(), Some("write"));
    assert!(metadata.has_tag("secured"));
    assert!(!metadata.is_empty());

    assert!(matches!(
        OperationMetadata::empty().with_tag("not stable"),
        Err(OperationMetadataError::InvalidTag { .. })
    ));
    assert!(matches!(
        OperationMetadata::empty().with_qualifier(""),
        Err(OperationMetadataError::InvalidQualifier { .. })
    ));
}

#[test]
fn operation_hash_identity_excludes_metadata_but_declaration_equality_keeps_it() {
    let plain = Operation::new("OrderService", "create");
    let declared = plain
        .clone()
        .with_tag("secured")
        .expect("valid declaration tag");
    let mut identities = HashSet::new();

    identities.insert(plain.clone());
    identities.insert(declared.clone());

    assert_eq!(plain, declared);
    assert!(!plain.same_declaration(&declared));
    assert_eq!(identities.len(), 1);
}

#[test]
fn tag_and_qualifier_pointcuts_compose_at_plan_compilation_time() {
    let operation = Operation::new("OrderService", "create")
        .with_tag("secured")
        .expect("valid declaration tag")
        .with_qualifier("write")
        .expect("valid declaration qualifier");
    let secured_write = TagPointcut::new("secured")
        .expect("valid pointcut tag")
        .and(QualifierPointcut::new("write").expect("valid pointcut qualifier"));

    assert!(secured_write.matches(&operation));
    assert!(
        !TagPointcut::new("internal")
            .expect("valid pointcut tag")
            .matches(&operation)
    );
    assert!(
        !QualifierPointcut::new("read")
            .expect("valid pointcut qualifier")
            .matches(&operation)
    );
}

#[tokio::test]
async fn send_plan_projects_declared_metadata_onto_runtime_identity() {
    let declaration = Operation::new("OrderService", "create")
        .with_tag("secured")
        .expect("valid declaration tag")
        .with_qualifier("write")
        .expect("valid declaration qualifier");
    let catalog = InvocationPlanBuilder::new()
        .build_catalog([declaration.clone()])
        .expect("metadata declaration should compile");
    let runtime_operation = Operation::new("OrderService", "create");
    let plan = catalog
        .get(&runtime_operation)
        .expect("runtime identity should find declared plan");
    let invocation = Invocation::new(runtime_operation).shared();
    let invocation_id = invocation.id();
    invocation.context().insert(7_u8).await;

    let target: Arc<InvocationTarget> = Arc::new(move |invocation| {
        Box::pin(async move {
            let observes_declaration = invocation.id() == invocation_id
                && invocation.operation().metadata().has_tag("secured")
                && invocation.operation().metadata().qualifier() == Some("write")
                && invocation.context().get::<u8>().await == Some(7);
            Ok(Box::new(observes_declaration) as InvocationValue)
        })
    });
    let observed = plan
        .invoke(invocation, target)
        .await
        .expect("declared operation should invoke")
        .downcast::<bool>()
        .expect("bool result");

    assert!(*observed);
    assert!(plan.operation().same_declaration(&declaration));
}

#[tokio::test(flavor = "current_thread")]
async fn local_plan_projects_the_same_declared_metadata() {
    let declaration = Operation::new("ActixOrderService", "create")
        .with_tag("worker-local")
        .expect("valid declaration tag");
    let catalog = LocalInvocationPlanBuilder::new()
        .build_catalog([declaration.clone()])
        .expect("local metadata declaration should compile");
    let runtime_operation = Operation::new("ActixOrderService", "create");
    let plan = catalog
        .get(&runtime_operation)
        .expect("runtime identity should find local declared plan");
    let invocation = Invocation::new(runtime_operation).shared();
    let target: Rc<LocalInvocationTarget> = Rc::new(|invocation| {
        Box::pin(async move {
            Ok(
                Box::new(invocation.operation().metadata().has_tag("worker-local"))
                    as LocalInvocationValue,
            )
        })
    });
    let observed = plan
        .invoke(invocation, target)
        .await
        .expect("declared local operation should invoke")
        .downcast::<bool>()
        .expect("bool result");

    assert!(*observed);
}

#[test]
fn send_and_local_catalogs_reject_conflicting_metadata_for_one_identity() {
    let secured = Operation::new("OrderService", "create")
        .with_tag("secured")
        .expect("valid declaration tag");
    let audited = Operation::new("OrderService", "create")
        .with_tag("audited")
        .expect("valid declaration tag");

    let Err(send_error) =
        InvocationPlanBuilder::new().build_catalog([secured.clone(), audited.clone()])
    else {
        panic!("Send catalog must reject conflicting metadata");
    };
    let Err(local_error) =
        LocalInvocationPlanBuilder::new().build_catalog([secured.clone(), audited.clone()])
    else {
        panic!("Local catalog must reject conflicting metadata");
    };

    assert_eq!(send_error.operation(), &secured);
    assert_eq!(send_error.existing(), secured.metadata());
    assert_eq!(send_error.duplicate(), audited.metadata());
    assert_eq!(local_error, send_error);
}
