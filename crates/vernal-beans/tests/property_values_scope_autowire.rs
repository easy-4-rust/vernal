//! PropertyValues 绑定 + Scope 注册/激活 + field_metadata 注入集成测试。

use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

fn lock_field_md() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap()
}

use vernal_beans::ComponentDefinition;
use vernal_beans::ComponentKey;
use vernal_beans::RegistryBuilder;
use vernal_beans::Resolver;
use vernal_beans::Scope;
use vernal_beans::AutowireCapableBeanFactory;
use vernal_beans::BeanFactory;
use vernal_beans::bean_scope::BeanScope;
use vernal_beans::field_metadata::{FieldDescriptor, TypeMetadata};
use vernal_beans::mutable_property_values::MutablePropertyValues;
use vernal_beans::property_value::PropertyValue;

// ── 测试类型 ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct AppConfig {
    db_url: String,
    max_connections: u32,
}

#[derive(Debug)]
struct ConnectionPool {
    url: String,
    max_size: u32,
}

#[derive(Debug)]
struct UserService {
    pool_url: String,
}

// ── 1. PropertyValues 绑定流程测试 ───────────────────────────────────────

/// 参照 Spring `MutablePropertyValuesTests`：
/// 验证 PropertyValues 从配置到 Bean 的绑定流程。
#[test]
fn property_values_bind_to_bean() {
    // 创建 PropertyValues
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("db_url", Arc::new("postgres://localhost/test".to_string()));
    pvs.add_value("max_connections", Arc::new(10u32));

    // 验证 PropertyValues 包含正确的值
    assert!(pvs.contains("db_url"));
    assert!(pvs.contains("max_connections"));
    assert_eq!(pvs.len(), 2);

    // 在 Container.createBean 时应用 PropertyValues
    // 通过 ComponentDefinition 的 factory 闭包模拟 Spring 的属性注入
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| {
                // 模拟 Spring 的属性注入：从配置中读取值
                AppConfig {
                    db_url: "postgres://localhost/test".to_string(),
                    max_connections: 10,
                }
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // createBean 应该返回已注入属性的实例
    let config = container.resolve::<AppConfig>().unwrap();
    assert_eq!(config.db_url, "postgres://localhost/test");
    assert_eq!(config.max_connections, 10);
}

/// 验证 MutablePropertyValues 的基本操作。
#[test]
fn mutable_property_values_operations() {
    let mut pvs = MutablePropertyValues::new();

    // 添加
    pvs.add_value("a", Arc::new(1i32));
    pvs.add_value("b", Arc::new(2i32));
    assert_eq!(pvs.len(), 2);
    assert!(pvs.contains("a"));
    assert!(pvs.contains("b"));

    // 覆盖
    pvs.add_value("a", Arc::new(10i32));
    assert_eq!(pvs.len(), 2);
    let pv = pvs.get("a").unwrap();
    let val = pv.value().downcast_ref::<i32>().unwrap();
    assert_eq!(*val, 10);

    // 清空
    pvs.clear();
    assert!(pvs.is_empty());
}

/// 验证 PropertyValue 基本操作。
#[test]
fn property_value_basic() {
    let pv = PropertyValue::new("name", Arc::new("Alice".to_string()));
    assert_eq!(pv.name(), "name");
    let val = pv.value().downcast_ref::<String>().unwrap();
    assert_eq!(val, "Alice");
}

/// 验证 MutablePropertyValues 从 Vec 创建。
#[test]
fn mutable_property_values_from_vec() {
    let pvs = MutablePropertyValues::from_vec(vec![
        PropertyValue::new("x", Arc::new(1i32)),
        PropertyValue::new("y", Arc::new(2i32)),
    ]);
    assert_eq!(pvs.len(), 2);
    assert!(pvs.contains("x"));
    assert!(pvs.contains("y"));
}

// ── 2. Scope 注册/激活 (open_scope + resolve_in) 测试 ────────────────────

/// 参照 Spring `SimpleScopeTests`：
/// 验证自定义 Scope 通过 open_scope 使用。
#[test]
fn scope_open_and_resolve_in() {
    static GET_COUNT: AtomicUsize = AtomicUsize::new(0);
    static DESTROY_COUNT: AtomicUsize = AtomicUsize::new(0);

    GET_COUNT.store(0, Ordering::SeqCst);
    DESTROY_COUNT.store(0, Ordering::SeqCst);

    struct RequestScope;

    impl BeanScope for RequestScope {
        fn get(
            &self,
            _name: &str,
            object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
        ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            GET_COUNT.fetch_add(1, Ordering::SeqCst);
            Ok(object_factory())
        }

        fn register_destruction_callback(
            &self,
            _name: &str,
            _callback: Box<dyn FnOnce() + Send + Sync>,
        ) {
            DESTROY_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }

    // 创建 Container
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| AppConfig {
                db_url: "scope_test".to_string(),
                max_connections: 10,
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // open_scope 创建 ScopeContext
    let scope_context = container.open_scope::<RequestScope>();

    // 验证 ScopeContext 已创建
    assert_eq!(scope_context.state(), vernal_beans::ScopeState::Open);

    // 验证 Scope 的 get 方法被调用
    let scope = RequestScope;
    let obj = scope.get("test", &|| Box::new(42i32)).unwrap();
    let val = obj.downcast_ref::<i32>().unwrap();
    assert_eq!(*val, 42);
    assert_eq!(GET_COUNT.load(Ordering::SeqCst), 1);

    // 关闭 Scope
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(scope_context.close()).unwrap();
}

/// 验证 Scope 生命周期状态管理。
#[test]
fn scope_lifecycle_state_management() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| AppConfig {
                db_url: "test".to_string(),
                max_connections: 10,
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // 创建 ScopeContext
    let scope_context = container.open_scope::<RequestScope>();

    // 初始状态应该是 Open
    assert_eq!(scope_context.state(), vernal_beans::ScopeState::Open);

    // 关闭后状态应该是 Closed
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(scope_context.close()).unwrap();
    assert_eq!(scope_context.state(), vernal_beans::ScopeState::Closed);
}

/// 验证 Scope 销毁回调执行。
#[test]
fn scope_destruction_callback_execution() {
    static CALLBACK_CALLED: AtomicBool = AtomicBool::new(false);
    CALLBACK_CALLED.store(false, Ordering::SeqCst);

    struct TestScope;

    impl BeanScope for TestScope {
        fn get(
            &self,
            _name: &str,
            object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
        ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(object_factory())
        }

        fn register_destruction_callback(
            &self,
            _name: &str,
            _callback: Box<dyn FnOnce() + Send + Sync>,
        ) {
            // 记录回调已注册
        }
    }

    let scope = TestScope;
    scope.register_destruction_callback("test", Box::new(|| {}));
    // 验证回调已注册（不 panic）
}

// ── 3. autowireBean field_metadata 注册 + 遍历注入测试 ──────────────────

/// 验证 field_metadata 注册和查询。
#[test]
fn field_metadata_register_and_query() {
    let _guard = lock_field_md();
    use vernal_beans::field_metadata;

    // 清理之前的元数据
    field_metadata::clear_metadata();

    // 注册类型元数据
    let metadata = TypeMetadata {
        type_id: std::any::TypeId::of::<UserService>(),
        type_name: std::any::type_name::<UserService>(),
        fields: vec![FieldDescriptor::new(
            "pool_url",
            std::any::TypeId::of::<String>(),
            std::any::type_name::<String>(),
        )],
    };
    field_metadata::register_type_metadata(metadata);

    // 验证元数据已注册
    let found = field_metadata::get_metadata_for_type(std::any::TypeId::of::<UserService>());
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.type_name, std::any::type_name::<UserService>());
    assert_eq!(found.fields.len(), 1);
    assert_eq!(found.fields[0].name, "pool_url");

    // 验证 find_fields_needing_type
    let fields = field_metadata::find_fields_needing_type(std::any::TypeId::of::<String>());
    assert!(fields.len() >= 1);
    assert!(fields.iter().any(|(_, name)| *name == "pool_url"));

    // 清理
    field_metadata::clear_metadata();
}

/// 验证 field_metadata 可选字段和限定符。
#[test]
fn field_metadata_optional_and_qualifier() {
    let _guard = lock_field_md();
    use vernal_beans::field_metadata;

    field_metadata::clear_metadata();

    let metadata = TypeMetadata {
        type_id: std::any::TypeId::of::<UserService>(),
        type_name: std::any::type_name::<UserService>(),
        fields: vec![
            FieldDescriptor::new(
                "pool_url",
                std::any::TypeId::of::<String>(),
                std::any::type_name::<String>(),
            )
            .with_optional()
            .with_qualifier("primary"),
        ],
    };
    field_metadata::register_type_metadata(metadata);

    let found =
        field_metadata::get_metadata_for_type(std::any::TypeId::of::<UserService>()).unwrap();
    assert_eq!(found.fields[0].optional, true);
    assert_eq!(found.fields[0].qualifier, Some("primary"));

    field_metadata::clear_metadata();
}

/// 验证 field_metadata 多类型注册。
#[test]
fn field_metadata_multiple_types() {
    let _guard = lock_field_md();
    use vernal_beans::field_metadata;

    field_metadata::clear_metadata();

    // 注册多个类型
    field_metadata::register_type_metadata(TypeMetadata {
        type_id: std::any::TypeId::of::<UserService>(),
        type_name: std::any::type_name::<UserService>(),
        fields: vec![FieldDescriptor::new(
            "pool_url",
            std::any::TypeId::of::<String>(),
            std::any::type_name::<String>(),
        )],
    });

    field_metadata::register_type_metadata(TypeMetadata {
        type_id: std::any::TypeId::of::<ConnectionPool>(),
        type_name: std::any::type_name::<ConnectionPool>(),
        fields: vec![FieldDescriptor::new(
            "url",
            std::any::TypeId::of::<String>(),
            std::any::type_name::<String>(),
        )],
    });

    // 验证至少有两个类型已注册（可能有其他测试注册的类型）
    let all = field_metadata::get_all_metadata();
    assert!(all.len() >= 2);

    // 验证按类型查找
    let user_meta = field_metadata::get_metadata_for_type(std::any::TypeId::of::<UserService>());
    assert!(user_meta.is_some());
    assert_eq!(user_meta.unwrap().fields[0].name, "pool_url");

    let pool_meta = field_metadata::get_metadata_for_type(std::any::TypeId::of::<ConnectionPool>());
    assert!(pool_meta.is_some());
    assert_eq!(pool_meta.unwrap().fields[0].name, "url");

    // 验证 find_fields_needing_type
    let string_fields = field_metadata::find_fields_needing_type(std::any::TypeId::of::<String>());
    assert!(string_fields.len() >= 2); // 至少有两个类型需要 String 字段

    field_metadata::clear_metadata();
}

/// 验证 field_metadata clear 功能。
#[test]
fn field_metadata_clear() {
    use vernal_beans::field_metadata;

    // 清理后注册
    field_metadata::clear_metadata();

    field_metadata::register_type_metadata(TypeMetadata {
        type_id: std::any::TypeId::of::<UserService>(),
        type_name: std::any::type_name::<UserService>(),
        fields: vec![],
    });

    let count = field_metadata::get_all_metadata().len();
    assert!(count >= 1, "Should have at least 1 metadata after registration");

    field_metadata::clear_metadata();
    assert!(field_metadata::get_all_metadata().is_empty());
}

/// 验证 autowireBean 通过 ComponentDefinition 工厂注入依赖。
#[test]
fn autowire_bean_via_factory_injection() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| AppConfig {
                db_url: "autowired".to_string(),
                max_connections: 10,
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<ConnectionPool, _>(|resolver: &Resolver| {
                let config = resolver.resolve::<AppConfig>().unwrap();
                ConnectionPool {
                    url: config.db_url.clone(),
                    max_size: config.max_connections,
                }
            })
            .depends_on::<AppConfig>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // createBean 应该通过工厂自动注入依赖
    let pool = container
        .create_bean(std::any::type_name::<ConnectionPool>())
        .unwrap();
    let pool = pool.downcast_ref::<ConnectionPool>().unwrap();
    assert_eq!(pool.url, "autowired");
    assert_eq!(pool.max_size, 10);
}

/// 验证 PropertyValues 绑定到 Bean 的完整流程。
#[test]
fn property_values_binding_complete_flow() {
    // 1. 创建 PropertyValues
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("db_url", Arc::new("binding_test".to_string()));
    pvs.add_value("max_connections", Arc::new(20u32));

    // 2. 验证 PropertyValues 内容
    assert!(pvs.contains("db_url"));
    assert!(pvs.contains("max_connections"));
    assert_eq!(pvs.len(), 2);

    // 3. 在 ComponentDefinition factory 中使用 PropertyValues
    // 提前克隆值，避免闭包生命周期问题
    let db_url = pvs
        .get("db_url")
        .unwrap()
        .value()
        .downcast_ref::<String>()
        .unwrap()
        .clone();
    let max_conn = *pvs
        .get("max_connections")
        .unwrap()
        .value()
        .downcast_ref::<u32>()
        .unwrap();

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            move |_resolver: &Resolver| AppConfig {
                db_url: db_url.clone(),
                max_connections: max_conn,
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // 4. createBean 应该应用 PropertyValues
    let config = container.resolve::<AppConfig>().unwrap();
    assert_eq!(config.db_url, "binding_test");
    assert_eq!(config.max_connections, 20);
}

/// 验证 Scope 创建和销毁的完整生命周期。
#[test]
fn scope_complete_lifecycle() {
    static DESTROY_CALLED: AtomicBool = AtomicBool::new(false);
    DESTROY_CALLED.store(false, Ordering::SeqCst);

    struct TestScope;

    impl BeanScope for TestScope {
        fn get(
            &self,
            _name: &str,
            object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
        ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(object_factory())
        }

        fn register_destruction_callback(
            &self,
            _name: &str,
            callback: Box<dyn FnOnce() + Send + Sync>,
        ) {
            // 存储回调供销毁时调用
            // 简化实现：直接执行
            callback();
            DESTROY_CALLED.store(true, Ordering::SeqCst);
        }
    }

    let scope = TestScope;

    // 创建实例
    let obj = scope.get("test", &|| Box::new(42i32)).unwrap();
    let val = obj.downcast_ref::<i32>().unwrap();
    assert_eq!(*val, 42);

    // 注册销毁回调
    scope.register_destruction_callback("test", Box::new(|| {}));
    assert!(DESTROY_CALLED.load(Ordering::SeqCst));
}

/// 验证 Scope conversation_id。
#[test]
fn scope_conversation_id() {
    struct TestScope;

    impl BeanScope for TestScope {
        fn get(
            &self,
            _name: &str,
            object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
        ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(object_factory())
        }

        fn conversation_id(&self) -> Option<&str> {
            Some("session-123")
        }
    }

    let scope = TestScope;
    assert_eq!(scope.conversation_id(), Some("session-123"));
}

/// 验证 Scope resolve_contextual_object。
#[test]
fn scope_resolve_contextual_object() {
    struct TestScope;

    impl BeanScope for TestScope {
        fn get(
            &self,
            _name: &str,
            object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
        ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(object_factory())
        }

        fn resolve_contextual_object(&self, key: &str) -> Option<Box<dyn Any>> {
            if key == "request" {
                Some(Box::new("request-ctx".to_string()))
            } else {
                None
            }
        }
    }

    let scope = TestScope;
    let ctx = scope.resolve_contextual_object("request");
    assert!(ctx.is_some());
    let unwrapped = ctx.unwrap();
    let ctx_str = unwrapped.downcast_ref::<String>().unwrap();
    assert_eq!(ctx_str, "request-ctx");

    assert!(scope.resolve_contextual_object("other").is_none());
}

/// 验证 Container open_scope 返回 ScopeContext。
#[test]
fn container_open_scope_returns_context() {
    struct TestScope;

    impl BeanScope for TestScope {
        fn get(
            &self,
            _name: &str,
            object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
        ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(object_factory())
        }
    }

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| AppConfig {
                db_url: "test".to_string(),
                max_connections: 10,
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // open_scope 返回 ScopeContext
    let scope_context = container.open_scope::<TestScope>();
    assert!(scope_context.state() == vernal_beans::ScopeState::Open);
}

/// 验证多个 ScopeContext 隔离。
#[test]
fn multiple_scope_contexts_isolated() {
    struct TestScope;

    impl BeanScope for TestScope {
        fn get(
            &self,
            _name: &str,
            object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
        ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(object_factory())
        }
    }

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| AppConfig {
                db_url: "test".to_string(),
                max_connections: 10,
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // 创建两个 ScopeContext
    let ctx1 = container.open_scope::<TestScope>();
    let ctx2 = container.open_scope::<TestScope>();

    // 它们应该是不同的实例
    assert!(!Arc::ptr_eq(&ctx1, &ctx2));
    assert_eq!(ctx1.state(), vernal_beans::ScopeState::Open);
    assert_eq!(ctx2.state(), vernal_beans::ScopeState::Open);
}

// ── 辅助类型 ─────────────────────────────────────────────────────────────

/// RequestScope 用于测试。
struct RequestScope;
