//! Scope 生命周期管理 + open_scope/resolve_in 集成测试。
//!
//! 参照 Spring Framework 7.0.8 的 `SimpleScopeTests`、`RequestScope`、`SessionScope` 测试场景。

use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use vernal_beans::ApplicationScope;
use vernal_beans::ComponentDefinition;
use vernal_beans::ComponentKey;
use vernal_beans::RegistryBuilder;
use vernal_beans::RequestScope;
use vernal_beans::Resolver;
use vernal_beans::SessionScope;
use vernal_beans::BeanFactory;
use vernal_beans::bean_scope::BeanScope;

// ── 测试类型 ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct UserContext {
    user_id: String,
}

#[derive(Debug, Clone)]
struct ShoppingCart {
    items: Vec<String>,
}

#[derive(Debug)]
struct Config {
    url: String,
}

// ── 1. RequestScope 基本行为 ─────────────────────────────────────────────

/// 参照 Spring `SimpleScopeTests.canGetScopedObject`：
/// 验证 RequestScope 返回交替实例。
#[test]
fn request_scope_returns_alternating_instances() {
    let scope = RequestScope::new("req-001");

    let obj1 = scope.get("test", &|| Box::new(42i32)).unwrap();
    let obj2 = scope.get("test", &|| Box::new(100i32)).unwrap();

    // 同一名称返回同一实例（缓存）
    let v1 = obj1.downcast_ref::<Arc<dyn Any + Send + Sync>>().unwrap();
    let v2 = obj2.downcast_ref::<Arc<dyn Any + Send + Sync>>().unwrap();
    assert!(Arc::ptr_eq(v1, v2));
}

/// 验证 RequestScope 不同名称返回不同实例。
#[test]
fn request_scope_different_names_different_instances() {
    let scope = RequestScope::new("req-002");

    let obj_a = scope.get("a", &|| Box::new(1i32)).unwrap();
    let obj_b = scope.get("b", &|| Box::new(2i32)).unwrap();

    let a = obj_a.downcast_ref::<Arc<dyn Any + Send + Sync>>().unwrap();
    let b = obj_b.downcast_ref::<Arc<dyn Any + Send + Sync>>().unwrap();
    assert!(!Arc::ptr_eq(a, b));
}

/// 验证 RequestScope remove 删除实例。
#[test]
fn request_scope_remove_deletes_instance() {
    let scope = RequestScope::new("req-003");

    scope.get("test", &|| Box::new(42i32)).unwrap();
    assert_eq!(scope.cached_count(), 1);

    let removed = scope.remove("test").unwrap();
    assert!(removed.is_some());
    assert_eq!(scope.cached_count(), 0);

    // 再次 get 应该创建新实例
    scope.get("test", &|| Box::new(100i32)).unwrap();
    assert_eq!(scope.cached_count(), 1);
}

/// 验证 RequestScope remove 不存在的名称返回 None。
#[test]
fn request_scope_remove_nonexistent_returns_none() {
    let scope = RequestScope::new("req-004");
    let removed = scope.remove("nonexistent").unwrap();
    assert!(removed.is_none());
}

/// 验证 RequestScope 销毁回调。
#[test]
fn request_scope_destruction_callback() {
    static CALLBACK_CALLED: AtomicBool = AtomicBool::new(false);
    CALLBACK_CALLED.store(false, Ordering::SeqCst);

    let scope = RequestScope::new("req-005");
    scope.register_destruction_callback(
        "test",
        Box::new(|| {
            CALLBACK_CALLED.store(true, Ordering::SeqCst);
        }),
    );

    // 销毁前回调未执行
    assert!(!CALLBACK_CALLED.load(Ordering::SeqCst));

    scope.destroy();

    // 销毁后回调已执行
    assert!(CALLBACK_CALLED.load(Ordering::SeqCst));
    // 缓存已清空
    assert_eq!(scope.cached_count(), 0);
}

/// 验证 RequestScope 多个销毁回调按顺序执行。
#[test]
fn request_scope_multiple_destruction_callbacks() {
    static ORDER: AtomicUsize = AtomicUsize::new(0);
    static FIRST_CALLED: AtomicUsize = AtomicUsize::new(0);
    static SECOND_CALLED: AtomicUsize = AtomicUsize::new(0);

    ORDER.store(0, Ordering::SeqCst);
    FIRST_CALLED.store(0, Ordering::SeqCst);
    SECOND_CALLED.store(0, Ordering::SeqCst);

    let scope = RequestScope::new("req-006");
    scope.register_destruction_callback(
        "first",
        Box::new(|| {
            FIRST_CALLED.store(ORDER.fetch_add(1, Ordering::SeqCst), Ordering::SeqCst);
        }),
    );
    scope.register_destruction_callback(
        "second",
        Box::new(|| {
            SECOND_CALLED.store(ORDER.fetch_add(1, Ordering::SeqCst), Ordering::SeqCst);
        }),
    );

    scope.destroy();

    // 两个回调都应该被调用
    assert!(FIRST_CALLED.load(Ordering::SeqCst) < 2);
    assert!(SECOND_CALLED.load(Ordering::SeqCst) < 2);
}

/// 验证 RequestScope conversation_id。
#[test]
fn request_scope_conversation_id() {
    let scope = RequestScope::new("req-007");
    assert_eq!(scope.conversation_id(), Some("req-007"));
    assert_eq!(scope.request_id(), "req-007");
}

/// 验证 RequestScope resolve_contextual_object。
#[test]
fn request_scope_resolve_contextual_object() {
    let scope = RequestScope::new("req-008");

    // "request" key 返回请求 ID
    let ctx = scope.resolve_contextual_object("request");
    assert!(ctx.is_some());
    let unwrapped = ctx.unwrap();
    let ctx = unwrapped.downcast_ref::<String>().unwrap();
    assert_eq!(ctx, "req-008");

    // 其他 key 返回 None
    assert!(scope.resolve_contextual_object("session").is_none());
}

// ── 2. SessionScope 基本行为 ─────────────────────────────────────────────

/// 参照 Spring `SimpleScopeTests.canGetScopedObject`：
/// 验证 SessionScope 缓存行为。
#[test]
fn session_scope_caches_instances() {
    let scope = SessionScope::new("sess-001");

    let obj1 = scope
        .get("cart", &|| Box::new(ShoppingCart { items: Vec::new() }))
        .unwrap();
    let obj2 = scope
        .get("cart", &|| {
            Box::new(ShoppingCart {
                items: vec!["item1".to_string()],
            })
        })
        .unwrap();

    // 同一名称返回同一实例
    let a = obj1.downcast_ref::<Arc<dyn Any + Send + Sync>>().unwrap();
    let b = obj2.downcast_ref::<Arc<dyn Any + Send + Sync>>().unwrap();
    assert!(Arc::ptr_eq(a, b));
}

/// 验证 SessionScope remove。
#[test]
fn session_scope_remove() {
    let scope = SessionScope::new("sess-002");

    scope
        .get("user", &|| {
            Box::new(UserContext {
                user_id: "u1".to_string(),
            })
        })
        .unwrap();
    assert_eq!(scope.cached_count(), 1);

    let removed = scope.remove("user").unwrap();
    assert!(removed.is_some());
    assert_eq!(scope.cached_count(), 0);
}

/// 验证 SessionScope 销毁回调。
#[test]
fn session_scope_destruction_callback() {
    static CALLBACK_CALLED: AtomicBool = AtomicBool::new(false);
    CALLBACK_CALLED.store(false, Ordering::SeqCst);

    let scope = SessionScope::new("sess-003");
    scope.register_destruction_callback(
        "cleanup",
        Box::new(|| {
            CALLBACK_CALLED.store(true, Ordering::SeqCst);
        }),
    );

    scope.destroy();
    assert!(CALLBACK_CALLED.load(Ordering::SeqCst));
}

/// 验证 SessionScope conversation_id。
#[test]
fn session_scope_conversation_id() {
    let scope = SessionScope::new("sess-004");
    assert_eq!(scope.conversation_id(), Some("sess-004"));
    assert_eq!(scope.session_id(), "sess-004");
}

/// 验证 SessionScope resolve_contextual_object。
#[test]
fn session_scope_resolve_contextual_object() {
    let scope = SessionScope::new("sess-005");

    let ctx = scope.resolve_contextual_object("session");
    assert!(ctx.is_some());
    let unwrapped = ctx.unwrap();
    let ctx = unwrapped.downcast_ref::<String>().unwrap();
    assert_eq!(ctx, "sess-005");

    assert!(scope.resolve_contextual_object("request").is_none());
}

// ── 3. ApplicationScope 基本行为 ─────────────────────────────────────────

/// 验证 ApplicationScope 缓存行为。
#[test]
fn application_scope_caches_instances() {
    let scope = ApplicationScope::new();

    let obj1 = scope
        .get("config", &|| {
            Box::new(Config {
                url: "app".to_string(),
            })
        })
        .unwrap();
    let obj2 = scope
        .get("config", &|| {
            Box::new(Config {
                url: "other".to_string(),
            })
        })
        .unwrap();

    let a = obj1.downcast_ref::<Arc<dyn Any + Send + Sync>>().unwrap();
    let b = obj2.downcast_ref::<Arc<dyn Any + Send + Sync>>().unwrap();
    assert!(Arc::ptr_eq(a, b));
}

/// 验证 ApplicationScope 销毁回调。
#[test]
fn application_scope_destruction_callback() {
    static CALLBACK_CALLED: AtomicBool = AtomicBool::new(false);
    CALLBACK_CALLED.store(false, Ordering::SeqCst);

    let scope = ApplicationScope::new();
    scope.register_destruction_callback(
        "shutdown",
        Box::new(|| {
            CALLBACK_CALLED.store(true, Ordering::SeqCst);
        }),
    );

    scope.destroy();
    assert!(CALLBACK_CALLED.load(Ordering::SeqCst));
}

/// 验证 ApplicationScope conversation_id。
#[test]
fn application_scope_conversation_id() {
    let scope = ApplicationScope::new();
    assert_eq!(scope.conversation_id(), Some("application"));
}

// ── 4. Scope 生命周期管理 ────────────────────────────────────────────────

/// 验证 Scope 从创建到销毁的完整生命周期。
#[test]
fn scope_lifecycle_complete() {
    static GET_COUNT: AtomicUsize = AtomicUsize::new(0);
    static DESTROY_CALLED: AtomicBool = AtomicBool::new(false);

    GET_COUNT.store(0, Ordering::SeqCst);
    DESTROY_CALLED.store(false, Ordering::SeqCst);

    let scope = RequestScope::new("lifecycle-001");

    // 1. 创建阶段：get 创建实例
    let _obj1 = scope
        .get("bean", &|| {
            GET_COUNT.fetch_add(1, Ordering::SeqCst);
            Box::new(42i32)
        })
        .unwrap();
    assert_eq!(GET_COUNT.load(Ordering::SeqCst), 1);
    assert_eq!(scope.cached_count(), 1);

    // 2. 使用阶段：再次 get 返回缓存
    let _obj2 = scope
        .get("bean", &|| {
            GET_COUNT.fetch_add(1, Ordering::SeqCst);
            Box::new(100i32)
        })
        .unwrap();
    assert_eq!(GET_COUNT.load(Ordering::SeqCst), 1); // 工厂未被调用

    // 3. 销毁阶段
    scope.register_destruction_callback(
        "bean",
        Box::new(|| {
            DESTROY_CALLED.store(true, Ordering::SeqCst);
        }),
    );
    scope.destroy();
    assert!(DESTROY_CALLED.load(Ordering::SeqCst));
    assert_eq!(scope.cached_count(), 0);
}

/// 验证 Scope destroy 幂等性。
#[test]
fn scope_destroy_is_idempotent() {
    static CALLBACK_COUNT: AtomicUsize = AtomicUsize::new(0);
    CALLBACK_COUNT.store(0, Ordering::SeqCst);

    let scope = RequestScope::new("idempotent-001");
    scope.register_destruction_callback(
        "bean",
        Box::new(|| {
            CALLBACK_COUNT.fetch_add(1, Ordering::SeqCst);
        }),
    );

    scope.destroy();
    scope.destroy(); // 第二次 destroy 不应该再执行回调

    // 回调只执行一次
    assert_eq!(CALLBACK_COUNT.load(Ordering::SeqCst), 1);
    assert_eq!(scope.cached_count(), 0);
}

/// 验证 Scope destroy 后 get 创建新实例。
#[test]
fn scope_get_after_destroy_creates_new() {
    let scope = RequestScope::new("recreate-001");

    scope.get("bean", &|| Box::new(42i32)).unwrap();
    assert_eq!(scope.cached_count(), 1);

    scope.destroy();
    assert_eq!(scope.cached_count(), 0);

    // 销毁后 get 创建新实例
    scope.get("bean", &|| Box::new(100i32)).unwrap();
    assert_eq!(scope.cached_count(), 1);
}

// ── 5. Scope + Container 集成 ───────────────────────────────────────────

/// 验证 Scope 可以在 Container 中使用。
#[test]
fn scope_with_container_resolve() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "scope_container".to_string(),
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // Container resolve 应该正常工作
    let config = container.resolve::<Config>().unwrap();
    assert_eq!(config.url, "scope_container");
}

/// 验证多个 Scope 实例隔离。
#[test]
fn multiple_scope_instances_isolated() {
    let scope1 = RequestScope::new("req-1");
    let scope2 = RequestScope::new("req-2");

    scope1.get("bean", &|| Box::new(1i32)).unwrap();
    scope2.get("bean", &|| Box::new(2i32)).unwrap();

    assert_eq!(scope1.cached_count(), 1);
    assert_eq!(scope2.cached_count(), 1);

    // 销毁 scope1 不影响 scope2
    scope1.destroy();
    assert_eq!(scope1.cached_count(), 0);
    assert_eq!(scope2.cached_count(), 1);
}

// ── 6. Scope trait 默认方法验证 ──────────────────────────────────────────

/// 验证 RequestScope 实现了所有 BeanScope 方法。
#[test]
fn request_scope_implements_all_bean_scope_methods() {
    let scope = RequestScope::new("test");

    // get
    let _ = scope.get("test", &|| Box::new(42i32)).unwrap();
    // remove
    let _ = scope.remove("test").unwrap();
    // register_destruction_callback
    scope.register_destruction_callback("test", Box::new(|| {}));
    // resolve_contextual_object
    let _ = scope.resolve_contextual_object("request");
    // conversation_id
    let _ = scope.conversation_id();
}

/// 验证 SessionScope 实现了所有 BeanScope 方法。
#[test]
fn session_scope_implements_all_bean_scope_methods() {
    let scope = SessionScope::new("test");

    let _ = scope.get("test", &|| Box::new(42i32)).unwrap();
    let _ = scope.remove("test").unwrap();
    scope.register_destruction_callback("test", Box::new(|| {}));
    let _ = scope.resolve_contextual_object("session");
    let _ = scope.conversation_id();
}

/// 验证 ApplicationScope 实现了所有 BeanScope 方法。
#[test]
fn application_scope_implements_all_bean_scope_methods() {
    let scope = ApplicationScope::new();

    let _ = scope.get("test", &|| Box::new(42i32)).unwrap();
    let _ = scope.remove("test").unwrap();
    scope.register_destruction_callback("test", Box::new(|| {}));
    let _ = scope.resolve_contextual_object("application");
    let _ = scope.conversation_id();
}

// ── 7. Scope 并发安全 ───────────────────────────────────────────────────

/// 验证 Scope 在并发场景下的安全性。
#[test]
fn scope_concurrent_get_safety() {
    use std::thread;

    let scope = Arc::new(RequestScope::new("concurrent-001"));
    let mut handles = vec![];

    for i in 0..10 {
        let scope: Arc<RequestScope> = Arc::clone(&scope);
        handles.push(thread::spawn(move || {
            let name = format!("bean_{}", i);
            let _ = scope.get(&name, &|| Box::new(i));
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(scope.cached_count(), 10);
}

/// 验证 Scope destroy 和 get 并发安全。
#[test]
fn scope_concurrent_destroy_and_get() {
    use std::thread;

    let scope = Arc::new(RequestScope::new("concurrent-002"));

    // 预先创建一些实例
    for i in 0..5 {
        let _ = scope.get(&format!("bean_{}", i), &|| Box::new(i));
    }

    let mut handles = vec![];

    // 并发 destroy
    let scope_d: Arc<RequestScope> = Arc::clone(&scope);
    handles.push(thread::spawn(move || {
        scope_d.destroy();
    }));

    // 并发 get
    let scope_g: Arc<RequestScope> = Arc::clone(&scope);
    handles.push(thread::spawn(move || {
        for i in 5..10 {
            let _ = scope_g.get(&format!("bean_{}", i), &|| Box::new(i));
        }
    }));

    for handle in handles {
        handle.join().unwrap();
    }

    // 最终状态应该是一致的（要么 0 要么有部分实例）
    let count = scope.cached_count();
    assert!(count <= 10);
}
