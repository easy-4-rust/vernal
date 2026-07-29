//! Container 实现 SingletonBeanRegistry trait 的集成测试。
//!
//! 对应 Spring 的 `SingletonBeanRegistry` 语义。

use std::any::Any;
use std::sync::Arc;
use vernal_beans::{
    BeanFactory, ComponentDefinition, ComponentKey, Container, RegistryBuilder,
    singleton_bean_registry::SingletonBeanRegistry,
};

fn make_container() -> Container {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }));
    Container::new(b.build().unwrap())
}

// ── register_singleton / get_singleton ─────────────────────────────

#[test]
fn sbr_register_and_get_singleton() {
    let c = make_container();

    // 手动注册一个单例
    c.register_singleton("myBean", Arc::new(42i32) as Arc<dyn Any + Send + Sync>);

    let instance = c.get_singleton("myBean");
    assert!(instance.is_some());
    let value = (*instance.unwrap()).downcast_ref::<i32>().copied().unwrap();
    assert_eq!(value, 42i32);
}

#[test]
fn sbr_get_singleton_not_found() {
    let c = make_container();
    assert!(c.get_singleton("nonExistent").is_none());
}

#[test]
fn sbr_get_singleton_via_once_lock() {
    // 通过 get_bean 解析后，singleton 应该可通过 get_singleton 获取
    let c = make_container();
    let _bean = c.get_bean_by_key(&ComponentKey::of::<String>()).unwrap();

    let instance = c.get_singleton(std::any::type_name::<String>());
    assert!(instance.is_some());
}

// ── contains_singleton ────────────────────────────────────────────

#[test]
fn sbr_contains_singleton_registered() {
    let c = make_container();
    c.register_singleton("test", Arc::new("value".to_string()));
    assert!(c.contains_singleton("test"));
}

#[test]
fn sbr_contains_singleton_not_found() {
    let c = make_container();
    assert!(!c.contains_singleton("nonExistent"));
}

#[test]
fn sbr_contains_singleton_via_once_lock() {
    let c = make_container();
    let _bean = c.get_bean_by_key(&ComponentKey::of::<String>()).unwrap();
    assert!(c.contains_singleton(std::any::type_name::<String>()));
}

// ── singleton_names / singleton_count ─────────────────────────────

#[test]
fn sbr_singleton_names_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(c.singleton_names().is_empty());
}

#[test]
fn sbr_singleton_count_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert_eq!(c.singleton_count(), 0);
}

#[test]
fn sbr_singleton_names_with_registered() {
    let c = make_container();
    c.register_singleton("customSingleton", Arc::new(99i32));

    let names = c.singleton_names();
    assert!(names.contains(&"customSingleton".to_string()));
}

#[test]
fn sbr_singleton_count_with_registered() {
    let c = make_container();
    c.register_singleton("s1", Arc::new(1i32));
    c.register_singleton("s2", Arc::new("two".to_string()));

    // count = registered (2) + empty once_lock cache (0 initially)
    assert_eq!(c.singleton_count(), 2);
}

#[test]
fn sbr_singleton_count_with_once_lock() {
    let c = make_container();
    let _bean = c.get_bean_by_key(&ComponentKey::of::<String>()).unwrap();

    c.register_singleton("custom", Arc::new(42i32));
    // count = registered (1) + once_lock (1)
    assert_eq!(c.singleton_count(), 2);
}

// ── add_singleton_callback ───────────────────────────────────────

#[test]
fn sbr_add_singleton_callback() {
    let mut c = make_container();

    use std::sync::atomic::{AtomicBool, Ordering};
    let called = Arc::new(AtomicBool::new(false));
    let called_clone = Arc::clone(&called);

    c.add_singleton_callback(
        "testBean".to_string(),
        Arc::new(move |_instance: &dyn Any| {
            called_clone.store(true, Ordering::SeqCst);
        }),
    );

    // 验证注册没有 panic
    // 注意：回调在单例创建时由 ApplicationContext 调用，Container 不自动调用
}

// ── singleton_mutex ──────────────────────────────────────────────

#[test]
fn sbr_singleton_mutex() {
    let c = make_container();
    let mutex = c.singleton_mutex();
    // 验证返回了一个可用的 Arc
    let _ = Arc::clone(&mutex);
}

#[test]
fn sbr_singleton_mutex_is_consistent() {
    let c = make_container();
    let m1 = c.singleton_mutex();
    let m2 = c.singleton_mutex();
    // 两次调用返回同一对象（Arc::ptr_eq 检查）
    assert!(Arc::ptr_eq(&m1, &m2));
}

// ── Replace existing singleton ───────────────────────────────────

#[test]
fn sbr_register_overwrites_existing() {
    let c = make_container();
    c.register_singleton("myBean", Arc::new("first".to_string()));
    c.register_singleton("myBean", Arc::new("second".to_string()));

    let instance = c.get_singleton("myBean").unwrap();
    let value = (*instance).downcast_ref::<String>().unwrap();
    assert_eq!(value, "second");
}

// ─── verify via BeanFactory integration ─────────────────────────

#[test]
fn sbr_integration_with_factory() {
    let c = make_container();

    // 注册一个单例
    c.register_singleton("myservice", Arc::new("my value".to_string()));

    // 可以通过 SingletonBeanRegistry 成功获取
    assert!(c.contains_singleton("myservice"));
    assert_eq!(c.singleton_count(), 1);
    let names = c.singleton_names();
    assert!(names.contains(&"myservice".to_string()));
}
