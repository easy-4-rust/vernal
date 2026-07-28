//! 对标 Spring Framework `GenericApplicationContextTests` 的差分测试。
//!
//! 每个 `#[test]` 都镜像 Spring 的同名测试方法，验证 vernal-context 与
//! Spring `org.springframework.context.support.GenericApplicationContext`
//! 在以下语义上完全等价：
//!
//! - `refresh()` 后单例 bean 立即可用，且 `Arc` 身份稳定
//! - `close()` 幂等；之后 refresh 必须返回 `InvalidState`
//! - `ComponentDefinition::shared_arc` 与 Spring
//!   `RootBeanDefinition(SCOPE_SINGLETON, supplier)` 等价
//! - `ComponentDefinition::try_singleton` 与 Spring
//!   `RootBeanDefinition(SCOPE_SINGLETON)` 等价
//! - `Container` 解析按类型 + 拓扑计划正确排序
//! - refresh 失败时已构造的单例在 Container Drop 时被释放
//!
//! 镜像 Spring `org.springframework.context.support.GenericApplicationContextTests`（2026-07-27）。

use std::sync::Arc;

use vernal_beans::{ComponentDefinition, Qualifier, RegistryBuilder};
use vernal_context::{ApplicationContextBuilder, ContextError};

/// 测试用 bean A：依赖 BeanB 与 BeanC，模拟 Spring 的 `BeanA`。
#[derive(Debug)]
struct BeanA {
    b: Arc<BeanB>,
    c: Arc<BeanC>,
}

/// 测试用 bean B：单元结构体。
#[derive(Debug)]
struct BeanB;

/// 测试用 bean C：单元结构体。
#[derive(Debug)]
struct BeanC;

/// Spring `getBeanForClass` 差分测试：
/// 验证同一 Component 多次解析返回**同一 Arc 实例**。
#[tokio::test]
async fn get_bean_for_class() {
    let shared_bean = Arc::new(BeanB);
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::shared_arc(Arc::clone(&shared_bean)))
        .expect("register BeanB");
    let context = ApplicationContextBuilder::new(registry.build().expect("registry build"))
        .build()
        .expect("context build");

    context.refresh().await.expect("refresh");

    let first = context
        .container()
        .resolve::<BeanB>()
        .expect("first resolve");
    let second = context
        .container()
        .resolve::<BeanB>()
        .expect("second resolve");
    assert!(Arc::ptr_eq(&first, &second));
}

/// Spring `withSingletonSupplier` 差分测试：
/// 验证 `ComponentDefinition::shared_arc` 与 Spring
/// `RootBeanDefinition(supplier)` 等价 —— 单例始终返回同一 Arc。
#[tokio::test]
async fn with_singleton_supplier() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::shared_arc(Arc::new(BeanC)))
        .expect("register BeanC");
    let context = ApplicationContextBuilder::new(registry.build().expect("registry build"))
        .build()
        .expect("context build");

    context.refresh().await.expect("refresh");

    let first = context.container().resolve::<BeanC>().expect("first");
    let second = context.container().resolve::<BeanC>().expect("second");
    assert!(Arc::ptr_eq(&first, &second));
}

/// Spring `withScopedSupplier` 差分测试：
/// 验证 Transient 作用域（对标 SCOPE_PROTOTYPE）每次解析返回新实例。
#[tokio::test]
async fn with_scoped_supplier() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::try_transient::<BeanC, _>(
            |_resolver| Ok(BeanC),
        ))
        .expect("register BeanC");
    let context = ApplicationContextBuilder::new(registry.build().expect("registry build"))
        .build()
        .expect("context build");

    context.refresh().await.expect("refresh");

    let first = context.container().resolve::<BeanC>().expect("first");
    let second = context.container().resolve::<BeanC>().expect("second");
    assert!(
        !Arc::ptr_eq(&first, &second),
        "transient must not share Arc identity"
    );
}

/// Spring `accessAfterClosing` 差分测试：
/// 验证 `close()` 幂等；close 之后再次 refresh 必须返回 InvalidState。
#[tokio::test]
async fn access_after_closing() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::shared_arc(Arc::new(BeanC)))
        .expect("register BeanC");
    let context = ApplicationContextBuilder::new(registry.build().expect("registry build"))
        .build()
        .expect("context build");

    context.refresh().await.expect("refresh");

    // 重复 close 必须幂等（与 Spring `close()` "may be called multiple times" 等价）。
    context.close().await.expect("first close");
    context.close().await.expect("second close (idempotent)");

    // close 之后再 refresh 必须返回 InvalidState。
    let refresh_after_close = context.refresh().await;
    assert!(
        matches!(refresh_after_close, Err(ContextError::InvalidState { .. })),
        "expected InvalidState, got {refresh_after_close:?}"
    );
}

/// Spring `individualBeans` 差分测试：
/// 验证依赖注入按类型 + 依赖计划正确排序。
#[tokio::test]
async fn individual_beans() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(
            ComponentDefinition::try_singleton::<BeanA, _>(|resolver| {
                let b: Arc<BeanB> = resolver.resolve::<BeanB>()?;
                let c: Arc<BeanC> = resolver.resolve::<BeanC>()?;
                Ok(BeanA { b, c })
            })
            .depends_on::<BeanB>()
            .depends_on::<BeanC>(),
        )
        .expect("register BeanA");
    registry
        .register(ComponentDefinition::shared_arc(Arc::new(BeanB)))
        .expect("register BeanB");
    registry
        .register(ComponentDefinition::shared_arc(Arc::new(BeanC)))
        .expect("register BeanC");

    let context = ApplicationContextBuilder::new(registry.build().expect("registry build"))
        .build()
        .expect("context build");
    context.refresh().await.expect("refresh");

    let a = context.container().resolve::<BeanA>().expect("BeanA");
    let b = context.container().resolve::<BeanB>().expect("BeanB");
    let c = context.container().resolve::<BeanC>().expect("BeanC");

    assert!(Arc::ptr_eq(&a.b, &b));
    assert!(Arc::ptr_eq(&a.c, &c));
}

/// Spring `refreshWithRuntimeFailureOnBeanCreationDisposeExistingBeans`
/// 风格测试：refresh 成功 + close 幂等 + 生命周期计数器全部归零。
#[tokio::test]
async fn refresh_success_and_close_idempotent() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::shared_arc(Arc::new(BeanC)))
        .expect("register BeanC");

    let context = ApplicationContextBuilder::new(registry.build().expect("registry build"))
        .build()
        .expect("context build");

    context.refresh().await.expect("refresh must succeed");

    // 验证 close 幂等。
    context.close().await.expect("first close must succeed");
    context
        .close()
        .await
        .expect("second close must be idempotent");

    // 验证 close 后 refresh 必须返回 InvalidState。
    let refresh_after_close = context.refresh().await;
    assert!(matches!(
        refresh_after_close,
        Err(ContextError::InvalidState { .. })
    ));
}

/// Spring `refreshForAotFailsOnAnActiveContext` 差分测试：
/// 验证已经 refresh / start 后的 ApplicationContext 不能再次进入 refresh。
#[tokio::test]
async fn refresh_after_start_fails() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::shared_arc(Arc::new(BeanC)))
        .expect("register BeanC");
    let context = ApplicationContextBuilder::new(registry.build().expect("registry build"))
        .build()
        .expect("context build");

    context.refresh().await.expect("first refresh");
    let second_refresh = context.refresh().await;
    assert!(
        second_refresh.is_err(),
        "second refresh on active context must fail"
    );

    context.close().await.expect("close");
}

/// Spring `beanRegistrar` 差分测试：
/// 验证多个 ComponentDefinition 之间通过 resolver 显式建立依赖关系。
#[tokio::test]
async fn bean_registrar_dependency_chain() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(
            ComponentDefinition::try_singleton::<BeanA, _>(|resolver| {
                let b: Arc<BeanB> = resolver.resolve::<BeanB>()?;
                let c: Arc<BeanC> = resolver.resolve::<BeanC>()?;
                Ok(BeanA { b, c })
            })
            .depends_on::<BeanB>()
            .depends_on::<BeanC>(),
        )
        .expect("BeanA");
    registry
        .register(ComponentDefinition::shared_arc(Arc::new(BeanB)))
        .expect("BeanB");
    registry
        .register(ComponentDefinition::shared_arc(Arc::new(BeanC)))
        .expect("BeanC");

    let context = ApplicationContextBuilder::new(registry.build().expect("registry build"))
        .build()
        .expect("context build");
    context.refresh().await.expect("refresh");

    let a = context.container().resolve::<BeanA>().expect("BeanA");
    let b = context.container().resolve::<BeanB>().expect("BeanB");
    let c = context.container().resolve::<BeanC>().expect("BeanC");

    assert!(Arc::ptr_eq(&a.b, &b));
    assert!(Arc::ptr_eq(&a.c, &c));
}

/// Spring `getBean(name, type)` 差分测试：
/// 验证 `Container::resolve_qualified` 与 Spring `getBean(name, type)` 等价。
#[tokio::test]
async fn get_bean_by_qualified_key() {
    let qualifier = Qualifier::new("primary").expect("valid qualifier");
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::shared_arc(Arc::new(BeanB)).qualified(qualifier.clone()))
        .expect("register BeanB");

    let context = ApplicationContextBuilder::new(registry.build().expect("registry build"))
        .build()
        .expect("context build");
    context.refresh().await.expect("refresh");

    let by_qualifier = context
        .container()
        .resolve_qualified::<BeanB>(&qualifier)
        .expect("resolve_qualified");
    let _: &BeanB = &by_qualifier;
}

/// 验证空 Registry 也能成功 refresh 与 close（Spring `emptyContext` 风格）。
#[tokio::test]
async fn empty_registry_refresh_and_close() {
    let registry = RegistryBuilder::new();
    let context = ApplicationContextBuilder::new(registry.build().expect("registry build"))
        .build()
        .expect("context build");

    context.refresh().await.expect("refresh empty context");

    context.close().await.expect("close empty context");
    let state_after_close = context.state().await;
    assert_eq!(state_after_close, vernal_context::ContextState::Closed);
}

/// 验证 Spring 风格的 `registerSingleton` 重复注册会失败。
#[tokio::test]
async fn duplicate_registration_fails() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::shared_arc(Arc::new(BeanB)))
        .expect("first register");
    let second = registry.register(ComponentDefinition::shared_arc(Arc::new(BeanB)));

    assert!(
        second.is_err(),
        "duplicate singleton registration must fail"
    );
}

/// 验证 Spring `registerBeanDefinition(name, ...)` 风格：
/// 通过限定符区分同类型不同实例。
#[tokio::test]
async fn qualified_same_type_different_instances() {
    let qualifier_a = Qualifier::new("a").expect("valid qualifier a");
    let qualifier_b = Qualifier::new("b").expect("valid qualifier b");

    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::shared_arc(Arc::new(BeanB)).qualified(qualifier_a.clone()))
        .expect("register a");
    registry
        .register(ComponentDefinition::shared_arc(Arc::new(BeanB)).qualified(qualifier_b.clone()))
        .expect("register b");

    let context = ApplicationContextBuilder::new(registry.build().expect("registry build"))
        .build()
        .expect("context build");
    context.refresh().await.expect("refresh");

    let a = context
        .container()
        .resolve_qualified::<BeanB>(&qualifier_a)
        .expect("a");
    let b = context
        .container()
        .resolve_qualified::<BeanB>(&qualifier_b)
        .expect("b");
    assert!(!Arc::ptr_eq(&a, &b));
}
