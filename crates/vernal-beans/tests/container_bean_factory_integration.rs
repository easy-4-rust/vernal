//! Container BeanFactory 集成测试。
//!
//! 参照 Spring Framework 7.0.8 的 `DefaultListableBeanFactoryTests`，
//! 使用 `RegistryBuilder` 注册真实 `ComponentDefinition` 并在 `Container` 中
//! resolve 验证语义正确性。
//!
//! 测试覆盖：
//! 1. Singleton 解析（同一容器返回同一实例）
//! 2. Prototype（Transient）解析（每次返回新实例）
//! 3. 依赖注入（depends_on）
//! 4. 循环依赖检测
//! 5. BeanFactory trait 方法（containsBean / isSingleton / isPrototype / getType）
//! 6. 已注册组件的 warm_up（预实例化所有 singleton）
//! 7. Registry 快照诊断

use std::sync::Arc;

use vernal_beans::ComponentDefinition;
use vernal_beans::ComponentKey;
use vernal_beans::RegistryBuilder;
use vernal_beans::Resolver;
use vernal_beans::Scope;
use vernal_beans::bean_factory::BeanFactory;

// ── 测试用类型 ───────────────────────────────────────────────────────────

#[derive(Debug)]
struct Config {
    db_url: String,
}

#[derive(Debug)]
struct DatabasePool {
    url: String,
}

#[derive(Debug)]
struct UserService {
    pool: Arc<DatabasePool>,
}

#[derive(Debug)]
struct OrderService {
    user_service: Arc<UserService>,
}

// ── 1. Singleton 解析 ────────────────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.unreferencedSingletonWasInstantiated`：
/// 验证 singleton 在 resolve 时创建并缓存。
#[test]
fn singleton_resolve_returns_same_instance() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(Config {
            db_url: "postgres://localhost/test".to_string(),
        }))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let a = container.resolve::<Config>().unwrap();
    let b = container.resolve::<Config>().unwrap();

    // Singleton 应返回同一个 Arc
    assert!(Arc::ptr_eq(&a, &b));
    assert_eq!(a.db_url, "postgres://localhost/test");
}

/// 参照 Spring `DefaultListableBeanFactoryTests.unreferencedSingletonWasInstantiated`：
/// 验证 singleton 工厂只调用一次。
#[test]
fn singleton_factory_called_once() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug)]
    struct SingletonService {
        id: usize,
    }

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<SingletonService, _>(
            |_resolver| {
                let id = CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                SingletonService { id }
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let a = container.resolve::<SingletonService>().unwrap();
    let b = container.resolve::<SingletonService>().unwrap();

    // 工厂只调用一次
    assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);
    // 两次返回同一个实例
    assert!(Arc::ptr_eq(&a, &b));
    assert_eq!(a.id, 0);
}

// ── 2. Prototype（Transient）解析 ────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.prototype`：
/// 验证 prototype 每次 resolve 返回新实例。
#[test]
fn transient_resolve_returns_new_instance() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static INSTANCE_COUNT: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug)]
    struct PrototypeService {
        id: usize,
    }

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::transient::<PrototypeService, _>(
            |_resolver| {
                let id = INSTANCE_COUNT.fetch_add(1, Ordering::SeqCst);
                PrototypeService { id }
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let a = container.resolve::<PrototypeService>().unwrap();
    let b = container.resolve::<PrototypeService>().unwrap();

    // 每次调用都创建新实例
    assert_eq!(INSTANCE_COUNT.load(Ordering::SeqCst), 2);
    // 两次返回不同的 Arc
    assert!(!Arc::ptr_eq(&a, &b));
    // 且 id 不同
    assert_ne!(a.id, b.id);
}

/// 参照 Spring `DefaultListableBeanFactoryTests.prototypeStringCreatedRepeatedly`：
/// 验证 prototype 字符串每次不同。
#[test]
fn transient_factory_called_each_time() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug)]
    struct UniqueValue {
        token: usize,
    }

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::transient::<UniqueValue, _>(
            |_resolver| UniqueValue {
                token: CALL_COUNT.fetch_add(1, Ordering::SeqCst),
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let a = container.resolve::<UniqueValue>().unwrap();
    let b = container.resolve::<UniqueValue>().unwrap();
    let c = container.resolve::<UniqueValue>().unwrap();

    assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 3);
    assert_ne!(a.token, b.token);
    assert_ne!(b.token, c.token);
}

// ── 3. 依赖注入（depends_on）─────────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.referenceByType`：
/// 验证 singleton 依赖注入。
#[test]
fn singleton_dependency_injection() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabasePool, _>(
            |_resolver| DatabasePool {
                url: "postgres://localhost/prod".to_string(),
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<UserService, _>(|resolver: &Resolver| {
                let pool = resolver.resolve::<DatabasePool>().unwrap();
                UserService { pool }
            })
            .depends_on::<DatabasePool>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let user_service = container.resolve::<UserService>().unwrap();
    assert_eq!(user_service.pool.url, "postgres://localhost/prod");

    // 验证 pool 是 singleton（同一个实例）
    let pool_a = container.resolve::<DatabasePool>().unwrap();
    let pool_b = container.resolve::<DatabasePool>().unwrap();
    assert!(Arc::ptr_eq(&pool_a, &pool_b));
}

/// 参照 Spring `DefaultListableBeanFactoryTests.referenceByName`：
/// 验证链式依赖注入。
#[test]
fn chained_dependency_injection() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabasePool, _>(
            |_resolver| DatabasePool {
                url: "postgres://localhost".to_string(),
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<UserService, _>(|resolver: &Resolver| {
                let pool = resolver.resolve::<DatabasePool>().unwrap();
                UserService { pool }
            })
            .depends_on::<DatabasePool>(),
        )
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<OrderService, _>(|resolver: &Resolver| {
                let user_service = resolver.resolve::<UserService>().unwrap();
                OrderService { user_service }
            })
            .depends_on::<UserService>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let order_service = container.resolve::<OrderService>().unwrap();
    assert_eq!(order_service.user_service.pool.url, "postgres://localhost");

    // 验证同一 pool 被共享
    let pool_direct = container.resolve::<DatabasePool>().unwrap();
    assert!(Arc::ptr_eq(&order_service.user_service.pool, &pool_direct));
}

// ── 4. 循环依赖检测 ──────────────────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.circularReferenceThroughAutowiring`：
/// 验证循环依赖检测。
#[test]
fn circular_dependency_detected() {
    #[derive(Debug)]
    struct ServiceA {
        _b: Arc<ServiceB>,
    }
    #[derive(Debug)]
    struct ServiceB {
        _a: Arc<ServiceA>,
    }

    let mut builder = RegistryBuilder::new();
    builder
        .register(
            ComponentDefinition::singleton::<ServiceA, _>(|resolver: &Resolver| {
                let b = resolver.resolve::<ServiceB>().unwrap();
                ServiceA { _b: b }
            })
            .depends_on::<ServiceB>(),
        )
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<ServiceB, _>(|resolver: &Resolver| {
                let a = resolver.resolve::<ServiceA>().unwrap();
                ServiceB { _a: a }
            })
            .depends_on::<ServiceA>(),
        )
        .unwrap();

    // 循环依赖在 build() 阶段（GraphPlanner::plan）检测
    let result = builder.build();
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Cycle") || err_msg.contains("cycle"));
}

// ── 5. BeanFactory trait 方法 ────────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.containsBeanReturnsTrueEvenForAbstractBeanDefinition`：
/// 验证 containsBean。
#[test]
fn bean_factory_contains_bean() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(|_resolver| {
            Config {
                db_url: "test".to_string(),
            }
        }))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let key = ComponentKey::of::<Config>();
    assert!(container.contains_bean(&key));

    let missing_key = ComponentKey::of::<UserService>();
    assert!(!container.contains_bean(&missing_key));
}

/// 参照 Spring `DefaultListableBeanFactoryTests.prototype`：
/// 验证 isSingleton / isPrototype。
#[test]
fn bean_factory_scope_query() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(|_resolver| {
            Config {
                db_url: "test".to_string(),
            }
        }))
        .unwrap();
    builder
        .register(
            ComponentDefinition::transient::<UserService, _>(|resolver: &Resolver| {
                let pool = resolver.resolve::<DatabasePool>().unwrap();
                UserService { pool }
            })
            .depends_on::<DatabasePool>(),
        )
        .unwrap();
    builder
        .register(ComponentDefinition::singleton::<DatabasePool, _>(
            |_resolver| DatabasePool {
                url: "test".to_string(),
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let config_key = ComponentKey::of::<Config>();
    assert!(container.is_singleton(&config_key).unwrap());
    assert!(!container.is_prototype(&config_key).unwrap());

    let user_key = ComponentKey::of::<UserService>();
    assert!(!container.is_singleton(&user_key).unwrap());
    assert!(container.is_prototype(&user_key).unwrap());
}

/// 验证 BeanFactory 的 FACTORY_BEAN_PREFIX 常量。
#[test]
fn bean_factory_prefix() {
    assert_eq!(vernal_beans::bean_factory::FACTORY_BEAN_PREFIX, "&");
}

// ── 6. warm_up 预实例化 ──────────────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.unreferencedSingletonWasInstantiated`：
/// 验证 warm_up 预实例化所有 singleton。
#[test]
fn warm_up_creates_all_singletons() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static POOL_CREATED: AtomicUsize = AtomicUsize::new(0);
    static CONFIG_CREATED: AtomicUsize = AtomicUsize::new(0);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(|_resolver| {
            CONFIG_CREATED.fetch_add(1, Ordering::SeqCst);
            Config {
                db_url: "warm".to_string(),
            }
        }))
        .unwrap();
    builder
        .register(ComponentDefinition::singleton::<DatabasePool, _>(
            |_resolver| {
                POOL_CREATED.fetch_add(1, Ordering::SeqCst);
                DatabasePool {
                    url: "warm".to_string(),
                }
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // warm_up 应该创建所有 singleton
    container.warm_up().unwrap();

    assert_eq!(CONFIG_CREATED.load(Ordering::SeqCst), 1);
    assert_eq!(POOL_CREATED.load(Ordering::SeqCst), 1);
}

/// 验证 warm_up 幂等性（多次调用不重复创建）。
#[test]
fn warm_up_is_idempotent() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(|_resolver| {
            CALL_COUNT.fetch_add(1, Ordering::SeqCst);
            Config {
                db_url: "test".to_string(),
            }
        }))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    container.warm_up().unwrap();
    container.warm_up().unwrap();
    container.warm_up().unwrap();

    // 工厂只调用一次
    assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);
}

// ── 7. Registry 快照诊断 ─────────────────────────────────────────────────

/// 验证 registry.snapshot() 返回正确的诊断信息。
#[test]
fn registry_snapshot_diagnostics() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(|_resolver| {
            Config {
                db_url: "test".to_string(),
            }
        }))
        .unwrap();
    builder
        .register(
            ComponentDefinition::transient::<UserService, _>(|resolver: &Resolver| {
                let pool = resolver.resolve::<DatabasePool>().unwrap();
                UserService { pool }
            })
            .depends_on::<DatabasePool>(),
        )
        .unwrap();
    builder
        .register(ComponentDefinition::singleton::<DatabasePool, _>(
            |_resolver| DatabasePool {
                url: "test".to_string(),
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();

    // 验证注册表统计
    assert_eq!(registry.len(), 3);

    let snapshot = registry.snapshot();
    let summary = snapshot.summary();
    assert_eq!(summary.definition_count(), 3);
    // Config 和 DatabasePool 是 singleton，UserService 是 transient
    assert_eq!(summary.singleton_count(), 2);
    assert_eq!(summary.transient_count(), 1);
}

// ── 8. 空容器 ────────────────────────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.empty`：
/// 验证空容器行为。
#[test]
fn empty_container_behavior() {
    let registry = vernal_beans::Registry::empty();
    let container = registry.container();

    // 空容器不应有任何组件
    let key = ComponentKey::of::<Config>();
    assert!(!container.contains_bean(&key));
    assert!(container.is_singleton(&key).is_err());
    assert!(container.is_prototype(&key).is_err());
    assert!(container.get_type(&key).is_err());

    // resolve 应该失败
    let result = container.resolve::<Config>();
    assert!(result.is_err());
}

// ── 9. 多实例歧义检测 ────────────────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.getBeanByTypeWithAmbiguity`：
/// 验证多实例歧义检测。
#[test]
fn ambiguous_type_detection() {
    // 注册两个相同类型的组件（通过 qualifier 区分）
    // 但如果不使用 qualifier，解析时应该报歧义
    // 注意：vernal-beans 的 ComponentKey 基于 TypeId，
    // 同一 TypeId 不能注册两次（会报 DuplicateDefinition）
    // 所以这里的歧义检测是通过 TraitBinding 实现的
    // 对于具体类型，同类型只能有一个定义

    // 验证重复注册会失败
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(|_resolver| {
            Config {
                db_url: "first".to_string(),
            }
        }))
        .unwrap();

    let result = builder.register(ComponentDefinition::singleton::<Config, _>(|_resolver| {
        Config {
            db_url: "second".to_string(),
        }
    }));

    // 应该报 DuplicateDefinition 错误
    assert!(result.is_err());
}

// ── 10. Qualifier 限定符解析 ─────────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.getBeanByTypeWithPrimary`：
/// 验证通过 qualifier 区分同类型的不同组件。
#[test]
fn qualified_bean_resolution() {
    use vernal_beans::Qualifier;

    let mut builder = RegistryBuilder::new();
    builder
        .register(
            ComponentDefinition::singleton::<Config, _>(|_resolver| Config {
                db_url: "primary_db".to_string(),
            })
            .qualified(Qualifier::new("primary").unwrap()),
        )
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<Config, _>(|_resolver| Config {
                db_url: "secondary_db".to_string(),
            })
            .qualified(Qualifier::new("secondary").unwrap()),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // 按 qualified 解析
    let primary = container
        .resolve_qualified::<Config>(&Qualifier::new("primary").unwrap())
        .unwrap();
    let secondary = container
        .resolve_qualified::<Config>(&Qualifier::new("secondary").unwrap())
        .unwrap();

    assert_eq!(primary.db_url, "primary_db");
    assert_eq!(secondary.db_url, "secondary_db");
    // 两个是不同的实例
    assert!(!Arc::ptr_eq(&primary, &secondary));
}

// ── 11. Transient 依赖的 Singleton 不传播 Scope ──────────────────────────

/// 参照 Spring 设计：Singleton 构造不会继承传入 Scope。
#[test]
fn singleton_scope_not_inherited() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabasePool, _>(
            |_resolver| DatabasePool {
                url: "singleton_pool".to_string(),
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::transient::<UserService, _>(|resolver: &Resolver| {
                let pool = resolver.resolve::<DatabasePool>().unwrap();
                UserService { pool }
            })
            .depends_on::<DatabasePool>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // 两次 resolve transient 的 UserService，pool 应该是同一个 singleton
    let a = container.resolve::<UserService>().unwrap();
    let b = container.resolve::<UserService>().unwrap();

    // pool 是 singleton，所以两次 UserService 中的 pool 是同一个
    assert!(Arc::ptr_eq(&a.pool, &b.pool));
    // 但 UserService 本身是 transient，所以是不同的实例
    assert!(!Arc::ptr_eq(&a, &b));
}

// ── 12. shared_value 注册 ────────────────────────────────────────────────

/// 验证 shared_value 注册的预构建实例。
#[test]
fn shared_value_registration() {
    let shared_config = Config {
        db_url: "shared_db".to_string(),
    };

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(shared_config))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let config = container.resolve::<Config>().unwrap();
    assert_eq!(config.db_url, "shared_db");
}

/// 验证 shared_arc 注册。
#[test]
fn shared_arc_registration() {
    let shared_pool = Arc::new(DatabasePool {
        url: "arc_pool".to_string(),
    });

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_arc(shared_pool.clone()))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let resolved = container.resolve::<DatabasePool>().unwrap();
    // 应该是同一个 Arc
    assert!(Arc::ptr_eq(&shared_pool, &resolved));
}

// ── 13. warm_up 失败时的错误传播 ─────────────────────────────────────────

/// 验证 warm_up 遇到工厂失败时返回错误。
#[test]
fn warm_up_failure_propagation() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::try_singleton::<Config, _>(
            |_resolver| Err("Database connection failed".into()),
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let result = container.warm_up();
    assert!(result.is_err());
}
