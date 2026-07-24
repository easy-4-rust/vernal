//! Vernal `IoC` 自定义 Scope SPI 合同测试。

#[path = "custom_scope_support/request_scope.rs"]
mod request_scope;
#[path = "custom_scope_support/request_service.rs"]
mod request_service;
#[path = "custom_scope_support/scoped_counter.rs"]
mod scoped_counter;
#[path = "custom_scope_support/slow_scoped.rs"]
mod slow_scoped;
#[path = "custom_scope_support/tenant_consumer.rs"]
mod tenant_consumer;
#[path = "custom_scope_support/tenant_scope.rs"]
mod tenant_scope;
#[path = "custom_scope_support/tenant_value.rs"]
mod tenant_value;

use std::{
    io,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    time::Duration,
};

use request_scope::RequestScope;
use request_service::RequestService;
use scoped_counter::ScopedCounter;
use slow_scoped::SlowScoped;
use tenant_consumer::TenantConsumer;
use tenant_scope::TenantScope;
use tenant_value::TenantValue;
use tokio::sync::Notify;
use vernal_ioc::{ComponentDefinition, RegistryBuilder, ResolveError, ScopeError, ScopeState};

#[tokio::test]
async fn custom_scope_requires_context_caches_once_and_isolates_siblings() {
    let calls = Arc::new(AtomicUsize::new(0));
    let observed_calls = Arc::clone(&calls);
    let mut builder = RegistryBuilder::new();
    builder
        .register(
            ComponentDefinition::scoped::<ScopedCounter, RequestScope, _>(move |_| ScopedCounter {
                sequence: observed_calls.fetch_add(1, Ordering::SeqCst),
            }),
        )
        .expect("scoped counter definition");
    let container = Arc::new(builder.build().expect("valid scoped graph").container());

    assert!(matches!(
        container.resolve::<ScopedCounter>(),
        Err(ResolveError::ScopeNotActive { .. })
    ));

    let first_scope = container.open_scope::<RequestScope>();
    let second_scope = container.open_scope::<RequestScope>();
    let first = container
        .resolve_in::<ScopedCounter>(&first_scope)
        .expect("first scoped resolution");
    let repeated = container
        .resolve_in::<ScopedCounter>(&first_scope)
        .expect("cached scoped resolution");
    let sibling = container
        .resolve_in::<ScopedCounter>(&second_scope)
        .expect("sibling scoped resolution");

    assert!(Arc::ptr_eq(&first, &repeated));
    assert!(!Arc::ptr_eq(&first, &sibling));
    assert_eq!(first.sequence, 0);
    assert_eq!(sibling.sequence, 1);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    let native = first_scope
        .get_or_insert_with(|| String::from("framework-native"))
        .expect("native scope object");
    let repeated_native = first_scope
        .get_or_insert_with(|| String::from("unreachable"))
        .expect("cached native scope object");
    assert!(Arc::ptr_eq(&native, &repeated_native));

    // 框架原生对象与 IoC 组件即使 Rust 类型相同也必须位于不同命名空间；
    // 否则 Adapter 写入请求对象可能劫持业务组件解析。
    let native_counter = first_scope
        .get_or_insert_with(|| ScopedCounter { sequence: 99 })
        .expect("same-type native object");
    assert_eq!(native_counter.sequence, 99);
    assert_eq!(first.sequence, 0);
    assert!(!Arc::ptr_eq(&native_counter, &first));

    // 同时覆盖相反顺序：先创建原生对象，再解析 IoC 组件。
    let native_first_scope = container.open_scope::<RequestScope>();
    let native_first = native_first_scope
        .get_or_insert_with(|| ScopedCounter { sequence: 100 })
        .expect("native object before component");
    let component_after_native = container
        .resolve_in::<ScopedCounter>(&native_first_scope)
        .expect("component after native object");
    assert_eq!(native_first.sequence, 100);
    assert_eq!(component_after_native.sequence, 2);
    assert!(!Arc::ptr_eq(&native_first, &component_after_native));

    first_scope.close().await.expect("first scope close");
    second_scope.close().await.expect("second scope close");
    native_first_scope
        .close()
        .await
        .expect("native-first scope close");
}

#[test]
fn same_scope_constructs_once_under_thread_concurrency() {
    let calls = Arc::new(AtomicUsize::new(0));
    let observed_calls = Arc::clone(&calls);
    let mut builder = RegistryBuilder::new();
    builder
        .register(
            ComponentDefinition::scoped::<ScopedCounter, RequestScope, _>(move |_| {
                std::thread::yield_now();
                ScopedCounter {
                    sequence: observed_calls.fetch_add(1, Ordering::SeqCst),
                }
            }),
        )
        .expect("scoped counter definition");
    let container = Arc::new(builder.build().expect("valid scoped graph").container());
    let scope = container.open_scope::<RequestScope>();

    let workers = (0..32)
        .map(|_| {
            let container = Arc::clone(&container);
            let scope = Arc::clone(&scope);
            std::thread::spawn(move || {
                container
                    .resolve_in::<ScopedCounter>(&scope)
                    .expect("concurrent scoped resolution")
            })
        })
        .collect::<Vec<_>>();
    let instances = workers
        .into_iter()
        .map(|worker| worker.join().expect("scope worker"))
        .collect::<Vec<_>>();

    for instance in &instances[1..] {
        assert!(Arc::ptr_eq(&instances[0], instance));
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn child_scope_resolves_parent_but_parent_cannot_capture_child() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::scoped::<TenantValue, TenantScope, _>(
            |_| TenantValue { name: "tenant-a" },
        ))
        .expect("tenant value definition");
    builder
        .register(
            ComponentDefinition::try_scoped::<RequestService, RequestScope, _>(|resolver| {
                Ok(RequestService {
                    tenant: resolver.resolve::<TenantValue>()?,
                })
            })
            .depends_on::<TenantValue>(),
        )
        .expect("request service definition");
    builder
        .register(
            ComponentDefinition::try_scoped::<TenantConsumer, TenantScope, _>(|resolver| {
                Ok(TenantConsumer {
                    _request: resolver.resolve::<RequestService>()?,
                })
            })
            .depends_on::<RequestService>(),
        )
        .expect("tenant consumer definition");
    let container = builder.build().expect("valid scope graph").container();
    let tenant = container.open_scope::<TenantScope>();
    let request = tenant.child::<RequestScope>();
    let sibling_request = tenant.child::<RequestScope>();

    let service = container
        .resolve_in::<RequestService>(&request)
        .expect("request can resolve parent tenant");
    let tenant_value = container
        .resolve_in::<TenantValue>(&tenant)
        .expect("tenant component");
    let sibling = container
        .resolve_in::<RequestService>(&sibling_request)
        .expect("sibling request component");

    assert_eq!(service.tenant.name, "tenant-a");
    assert!(Arc::ptr_eq(&service.tenant, &tenant_value));
    assert!(Arc::ptr_eq(&sibling.tenant, &tenant_value));
    assert!(!Arc::ptr_eq(&service, &sibling));
    let Err(error) = container.resolve_in::<TenantConsumer>(&request) else {
        panic!("parent scope must not capture child component");
    };
    let ResolveError::Construction { source, .. } = error else {
        panic!("scope narrowing failure should preserve construction boundary");
    };
    assert!(matches!(
        source.as_ref().downcast_ref::<ResolveError>(),
        Some(ResolveError::ScopeNotActive { .. })
    ));

    request.close().await.expect("request close");
    sibling_request
        .close()
        .await
        .expect("sibling request close");
    tenant.close().await.expect("tenant close");
}

#[tokio::test]
async fn close_runs_all_hooks_in_reverse_and_rejects_late_resolution() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(
            ComponentDefinition::scoped::<ScopedCounter, RequestScope, _>(|_| ScopedCounter {
                sequence: 7,
            }),
        )
        .expect("scoped counter definition");
    let container = builder.build().expect("valid scoped graph").container();
    let scope = container.open_scope::<RequestScope>();
    let order = Arc::new(Mutex::new(Vec::new()));

    for (value, fail) in [(1, false), (2, true), (3, false)] {
        let order = Arc::clone(&order);
        scope
            .on_close(move || async move {
                order.lock().expect("order lock").push(value);
                if fail {
                    Err(io::Error::other("close failed"))
                } else {
                    Ok(())
                }
            })
            .expect("close hook registration");
    }
    container
        .resolve_in::<ScopedCounter>(&scope)
        .expect("component before close");

    let (first_close, second_close) = tokio::join!(scope.close(), scope.close());
    assert!(matches!(first_close, Err(ScopeError::CloseHook { .. })));
    assert!(matches!(second_close, Err(ScopeError::CloseHook { .. })));
    assert_eq!(*order.lock().expect("order lock"), vec![3, 2, 1]);
    assert_eq!(scope.state(), ScopeState::Closed);
    assert!(scope.cancellation().is_cancelled());
    assert!(matches!(
        container.resolve_in::<ScopedCounter>(&scope),
        Err(ResolveError::ScopeUnavailable {
            state: ScopeState::Closed,
            ..
        })
    ));
    assert!(matches!(
        scope.get_or_insert_with(|| String::from("late")),
        Err(ScopeError::InvalidState {
            state: ScopeState::Closed,
            ..
        })
    ));
    assert!(matches!(
        scope.close().await,
        Err(ScopeError::CloseHook { .. })
    ));
}

#[tokio::test]
async fn scope_cannot_cross_container_and_parent_cancel_stops_child() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(
            ComponentDefinition::scoped::<ScopedCounter, RequestScope, _>(|_| ScopedCounter {
                sequence: 1,
            }),
        )
        .expect("scoped counter definition");
    let registry = builder.build().expect("valid scoped graph");
    let first = registry.container();
    let second = registry.container();
    let tenant = first.open_scope::<TenantScope>();
    let request = tenant.child::<RequestScope>();

    assert!(matches!(
        second.resolve_in::<ScopedCounter>(&request),
        Err(ResolveError::ScopeOwnerMismatch { .. })
    ));
    tenant.close().await.expect("parent close");
    assert!(matches!(
        first.resolve_in::<ScopedCounter>(&request),
        Err(ResolveError::ScopeUnavailable {
            state: ScopeState::Open,
            cancelled: true,
            ..
        })
    ));
    request.close().await.expect("cancelled child still closes");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn close_waits_for_factory_that_started_while_scope_was_open() {
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Arc::new(Mutex::new(release_rx));
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::scoped::<SlowScoped, RequestScope, _>(
            move |_| {
                started_tx.send(()).expect("signal factory start");
                release_rx
                    .lock()
                    .expect("release lock")
                    .recv()
                    .expect("factory release");
                SlowScoped
            },
        ))
        .expect("slow scoped definition");
    let container = Arc::new(builder.build().expect("valid scoped graph").container());
    let scope = container.open_scope::<RequestScope>();

    let resolution = {
        let container = Arc::clone(&container);
        let scope = Arc::clone(&scope);
        tokio::task::spawn_blocking(move || container.resolve_in::<SlowScoped>(&scope))
    };
    tokio::task::spawn_blocking(move || started_rx.recv().expect("factory started"))
        .await
        .expect("start observer");

    let closing = {
        let scope = Arc::clone(&scope);
        tokio::spawn(async move { scope.close().await })
    };
    tokio::task::yield_now().await;
    assert!(!closing.is_finished());
    release_tx.send(()).expect("release slow factory");

    resolution
        .await
        .expect("resolution task")
        .expect("started factory may finish");
    closing
        .await
        .expect("close task")
        .expect("scope closes after factory");
    assert_eq!(scope.state(), ScopeState::Closed);
}

#[tokio::test]
async fn cancelling_one_close_waiter_never_cancels_scope_cleanup() {
    let scope = RegistryBuilder::new()
        .build()
        .expect("empty registry")
        .container()
        .open_scope::<RequestScope>();
    let entered = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let completed = Arc::new(AtomicUsize::new(0));

    let tail_completed = Arc::clone(&completed);
    scope
        .on_close(move || async move {
            tail_completed.fetch_add(1, Ordering::SeqCst);
            Ok::<_, io::Error>(())
        })
        .expect("tail hook");
    let blocking_entered = Arc::clone(&entered);
    let blocking_release = Arc::clone(&release);
    let blocking_completed = Arc::clone(&completed);
    scope
        .on_close(move || async move {
            blocking_entered.notify_one();
            blocking_release.notified().await;
            blocking_completed.fetch_add(1, Ordering::SeqCst);
            Ok::<_, io::Error>(())
        })
        .expect("blocking hook");

    let waiter = {
        let scope = Arc::clone(&scope);
        tokio::spawn(async move { scope.close().await })
    };
    entered.notified().await;
    waiter.abort();
    assert!(
        waiter
            .await
            .expect_err("waiter must be cancelled")
            .is_cancelled()
    );
    assert_eq!(scope.state(), ScopeState::Closing);

    release.notify_one();
    scope.close().await.expect("shared cleanup result");
    assert_eq!(completed.load(Ordering::SeqCst), 2);
    assert_eq!(scope.state(), ScopeState::Closed);
}

#[tokio::test]
async fn close_timeout_leaves_cleanup_running_and_allows_later_join() {
    let scope = RegistryBuilder::new()
        .build()
        .expect("empty registry")
        .container()
        .open_scope::<RequestScope>();
    let release = Arc::new(Notify::new());
    let hook_release = Arc::clone(&release);
    scope
        .on_close(move || async move {
            hook_release.notified().await;
            Ok::<_, io::Error>(())
        })
        .expect("blocking hook");

    let error = scope
        .close_with_timeout(Duration::from_millis(1))
        .await
        .expect_err("bounded wait must time out");
    assert!(matches!(error, ScopeError::CloseTimeout { .. }));
    assert_eq!(scope.state(), ScopeState::Closing);

    release.notify_one();
    scope.close().await.expect("later waiter joins cleanup");
    assert_eq!(scope.state(), ScopeState::Closed);
}

#[tokio::test]
async fn panicking_close_hook_does_not_skip_remaining_hooks() {
    let scope = RegistryBuilder::new()
        .build()
        .expect("empty registry")
        .container()
        .open_scope::<RequestScope>();
    let completed = Arc::new(AtomicUsize::new(0));
    let remaining_completed = Arc::clone(&completed);
    scope
        .on_close(move || async move {
            remaining_completed.fetch_add(1, Ordering::SeqCst);
            Ok::<_, io::Error>(())
        })
        .expect("remaining hook");
    scope
        .on_close(|| async {
            panic!("test close hook panic");
            #[allow(unreachable_code)]
            Ok::<_, io::Error>(())
        })
        .expect("panicking hook");

    assert!(matches!(
        scope.close().await,
        Err(ScopeError::CloseTask { .. })
    ));
    assert_eq!(completed.load(Ordering::SeqCst), 1);
    assert_eq!(scope.state(), ScopeState::Closed);
}
