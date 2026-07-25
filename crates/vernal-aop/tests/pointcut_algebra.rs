//! Vernal 内建切点与逻辑组合合同测试。

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use vernal_aop::{
    AnyPointcut, ComponentPointcut, MethodPointcut, Operation, OperationPointcut, Pointcut,
    PointcutExt,
};

#[test]
fn built_in_pointcuts_match_only_their_declared_operation_dimension() {
    let create_order = Operation::new("OrderService", "create");
    let cancel_order = Operation::new("OrderService", "cancel");
    let create_user = Operation::new("UserService", "create");

    assert!(AnyPointcut::new().matches(&create_order));
    assert!(OperationPointcut::new(create_order.clone()).matches(&create_order));
    assert!(!OperationPointcut::new(create_order.clone()).matches(&cancel_order));
    assert!(ComponentPointcut::new("OrderService").matches(&cancel_order));
    assert!(!ComponentPointcut::new("OrderService").matches(&create_user));
    assert!(MethodPointcut::new("create").matches(&create_user));
    assert!(!MethodPointcut::new("create").matches(&cancel_order));
}

#[test]
fn and_or_and_not_form_a_short_circuiting_pointcut_algebra() {
    let order_mutations = ComponentPointcut::new("OrderService")
        .and(MethodPointcut::new("create").or(MethodPointcut::new("cancel")));
    let not_health = OperationPointcut::new(Operation::new("System", "health")).not();
    let guarded = order_mutations.and(not_health);

    assert!(guarded.matches(&Operation::new("OrderService", "create")));
    assert!(guarded.matches(&Operation::new("OrderService", "cancel")));
    assert!(!guarded.matches(&Operation::new("OrderService", "find")));
    assert!(!guarded.matches(&Operation::new("System", "health")));
}

#[test]
fn existing_closure_pointcuts_gain_the_same_composition_methods() {
    let public_component = |operation: &Operation| operation.component().starts_with("Public");
    let read_method = |operation: &Operation| operation.method().starts_with("get");
    let pointcut = public_component.and(read_method);

    assert!(pointcut.matches(&Operation::new("PublicCatalog", "get_item")));
    assert!(!pointcut.matches(&Operation::new("InternalCatalog", "get_item")));
    assert!(!pointcut.matches(&Operation::new("PublicCatalog", "delete_item")));
}

#[test]
fn logical_combinators_do_not_evaluate_an_unneeded_branch() {
    let evaluations = Arc::new(AtomicUsize::new(0));
    let and_evaluations = Arc::clone(&evaluations);
    let or_evaluations = Arc::clone(&evaluations);
    let operation = Operation::new("OrderService", "create");

    let never_reached_by_and = move |_operation: &Operation| {
        and_evaluations.fetch_add(1, Ordering::Relaxed);
        true
    };
    let never_reached_by_or = move |_operation: &Operation| {
        or_evaluations.fetch_add(1, Ordering::Relaxed);
        false
    };

    assert!(
        !OperationPointcut::new(Operation::new("OtherService", "create"))
            .and(never_reached_by_and)
            .matches(&operation)
    );
    assert!(
        AnyPointcut::new()
            .or(never_reached_by_or)
            .matches(&operation)
    );
    assert_eq!(evaluations.load(Ordering::Relaxed), 0);
}
