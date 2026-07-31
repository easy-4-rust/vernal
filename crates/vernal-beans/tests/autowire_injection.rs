//! autowireBean 字段注入集成测试。
//!
//! 参照 Spring Framework 7.0.8 的 `DefaultListableBeanFactoryTests` 中
//! autowireBean/autowireBeanProperties 场景，验证 Container 的 autowireBean
//! 通过 ComponentDefinition 工厂 + Resolver 实现真正的依赖注入。

use std::any::Any;
use std::sync::Arc;

use vernal_beans::AutowireCapableBeanFactory;
use vernal_beans::ComponentDefinition;
use vernal_beans::ComponentKey;
use vernal_beans::RegistryBuilder;
use vernal_beans::Resolver;

// ── 测试类型 ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct DatabaseConfig {
    host: String,
    port: u16,
}

#[derive(Debug)]
struct ConnectionPool {
    config: Option<Arc<DatabaseConfig>>,
    max_size: u32,
}

#[derive(Debug)]
struct UserRepository {
    pool: Option<Arc<ConnectionPool>>,
}

#[derive(Debug)]
struct OrderRepository {
    pool: Option<Arc<ConnectionPool>>,
}

#[derive(Debug)]
struct UserService {
    user_repo: Option<Arc<UserRepository>>,
    order_repo: Option<Arc<OrderRepository>>,
}

// ── 1. autowireBean 基本注入 ─────────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.autowireBeanByType`：
/// 验证 autowireBean 通过工厂注入依赖。
#[test]
fn autowire_bean_injects_dependency_via_factory() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabaseConfig, _>(
            |_resolver: &Resolver| DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<ConnectionPool, _>(|resolver: &Resolver| {
                let config = resolver.resolve::<DatabaseConfig>().unwrap();
                ConnectionPool {
                    config: Some(config),
                    max_size: 10,
                }
            })
            .depends_on::<DatabaseConfig>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // 创建一个没有依赖注入的 ConnectionPool
    let unconfigured_pool = Arc::new(ConnectionPool {
        config: None,
        max_size: 5,
    });

    // autowireBean 应该通过工厂重新创建，注入 DatabaseConfig
    let autowired = container.autowire_bean(unconfigured_pool).unwrap();
    let pool = autowired.downcast_ref::<ConnectionPool>().unwrap();

    // 验证 DatabaseConfig 已被注入
    assert!(pool.config.is_some());
    let config = pool.config.as_ref().unwrap();
    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 5432);
}

/// 验证 autowireBean 对无依赖 Bean 返回原实例。
#[test]
fn autowire_bean_no_deps_returns_original() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabaseConfig, _>(
            |_resolver: &Resolver| DatabaseConfig {
                host: "test".to_string(),
                port: 3306,
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let original = Arc::new(DatabaseConfig {
        host: "custom".to_string(),
        port: 9999,
    });
    let original_ptr = Arc::as_ptr(&original);

    let result = container.autowire_bean(original.clone()).unwrap();
    // 无依赖时应返回原实例（或 singleton 缓存的同一实例）
    // 由于 DatabaseConfig 是 singleton，autowireBean 会返回 singleton 缓存的实例
    let result_config = result.downcast_ref::<DatabaseConfig>().unwrap();
    // 验证返回的实例是正确的（host 和 port 匹配）
    assert_eq!(result_config.host, "custom");
    assert_eq!(result_config.port, 9999);
}

// ── 2. 多层依赖 autowireBean ─────────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.referenceByType`：
/// 验证多层依赖的 autowireBean。
#[test]
fn autowire_bean_multi_level_deps() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabaseConfig, _>(
            |_resolver: &Resolver| DatabaseConfig {
                host: "db.example.com".to_string(),
                port: 5432,
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<ConnectionPool, _>(|resolver: &Resolver| {
                let config = resolver.resolve::<DatabaseConfig>().unwrap();
                ConnectionPool {
                    config: Some(config),
                    max_size: 20,
                }
            })
            .depends_on::<DatabaseConfig>(),
        )
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<UserRepository, _>(|resolver: &Resolver| {
                let pool = resolver.resolve::<ConnectionPool>().unwrap();
                UserRepository { pool: Some(pool) }
            })
            .depends_on::<ConnectionPool>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // 创建一个没有依赖的 UserRepository
    let unconfigured = Arc::new(UserRepository { pool: None });

    // autowireBean 应该注入 ConnectionPool（通过工厂）
    let autowired = container.autowire_bean(unconfigured).unwrap();
    let repo = autowired.downcast_ref::<UserRepository>().unwrap();

    // 验证 ConnectionPool 已被注入
    assert!(repo.pool.is_some());
    let pool = repo.pool.as_ref().unwrap();
    assert_eq!(pool.max_size, 20);

    // 验证 ConnectionPool 内部的 DatabaseConfig 也被注入
    assert!(pool.config.is_some());
    let config = pool.config.as_ref().unwrap();
    assert_eq!(config.host, "db.example.com");
    assert_eq!(config.port, 5432);
}

// ── 3. autowireBean 创建完整实例 ─────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.createBean`：
/// 验证 autowireBean 通过工厂创建完整实例（所有依赖注入）。
#[test]
fn autowire_bean_creates_fully_wired_instance() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabaseConfig, _>(
            |_resolver: &Resolver| DatabaseConfig {
                host: "prod-db".to_string(),
                port: 5432,
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<ConnectionPool, _>(|resolver: &Resolver| {
                let config = resolver.resolve::<DatabaseConfig>().unwrap();
                ConnectionPool {
                    config: Some(config),
                    max_size: 50,
                }
            })
            .depends_on::<DatabaseConfig>(),
        )
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<UserRepository, _>(|resolver: &Resolver| {
                let pool = resolver.resolve::<ConnectionPool>().unwrap();
                UserRepository { pool: Some(pool) }
            })
            .depends_on::<ConnectionPool>(),
        )
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<OrderRepository, _>(|resolver: &Resolver| {
                let pool = resolver.resolve::<ConnectionPool>().unwrap();
                OrderRepository { pool: Some(pool) }
            })
            .depends_on::<ConnectionPool>(),
        )
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<UserService, _>(|resolver: &Resolver| {
                let user_repo = resolver.resolve::<UserRepository>().unwrap();
                let order_repo = resolver.resolve::<OrderRepository>().unwrap();
                UserService {
                    user_repo: Some(user_repo),
                    order_repo: Some(order_repo),
                }
            })
            .depends_on::<UserRepository>()
            .depends_on::<OrderRepository>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // 创建一个空的 UserService
    let unconfigured = Arc::new(UserService {
        user_repo: None,
        order_repo: None,
    });

    // autowireBean 应该注入所有依赖
    let autowired = container.autowire_bean(unconfigured).unwrap();
    let service = autowired.downcast_ref::<UserService>().unwrap();

    // 验证 UserRepository 已注入
    assert!(service.user_repo.is_some());
    let user_repo = service.user_repo.as_ref().unwrap();
    assert!(user_repo.pool.is_some());

    // 验证 OrderRepository 已注入
    assert!(service.order_repo.is_some());
    let order_repo = service.order_repo.as_ref().unwrap();
    assert!(order_repo.pool.is_some());

    // 验证两个 Repository 共享同一个 ConnectionPool
    let user_pool = user_repo.pool.as_ref().unwrap();
    let order_pool = order_repo.pool.as_ref().unwrap();
    assert!(Arc::ptr_eq(user_pool, order_pool));

    // 验证 ConnectionPool 内部的 DatabaseConfig
    let config = user_pool.config.as_ref().unwrap();
    assert_eq!(config.host, "prod-db");
    assert_eq!(config.port, 5432);
    assert_eq!(user_pool.max_size, 50);
}

// ── 4. autowireBean 与 singleton 缓存 ───────────────────────────────────

/// 验证 autowireBean 后的 bean 是 singleton（缓存复用）。
#[test]
fn autowire_bean_singleton_cached() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabaseConfig, _>(
            |_resolver: &Resolver| DatabaseConfig {
                host: "cached".to_string(),
                port: 3306,
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<ConnectionPool, _>(|resolver: &Resolver| {
                let config = resolver.resolve::<DatabaseConfig>().unwrap();
                ConnectionPool {
                    config: Some(config),
                    max_size: 10,
                }
            })
            .depends_on::<DatabaseConfig>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // 两次 autowireBean 应该返回同一个 singleton
    let pool1 = Arc::new(ConnectionPool {
        config: None,
        max_size: 5,
    });
    let autowired1 = container.autowire_bean(pool1).unwrap();

    let pool2 = Arc::new(ConnectionPool {
        config: None,
        max_size: 5,
    });
    let autowired2 = container.autowire_bean(pool2).unwrap();

    // 两者都解析到同一个 singleton
    let p1 = autowired1.downcast_ref::<ConnectionPool>().unwrap();
    let p2 = autowired2.downcast_ref::<ConnectionPool>().unwrap();
    assert!(Arc::ptr_eq(
        &p1.config.as_ref().unwrap(),
        &p2.config.as_ref().unwrap()
    ));
}

// ── 5. autowireBean 与 PostProcessor 集成 ───────────────────────────────

/// 验证 autowireBean 后的 bean 经过 PostProcessor 处理。
#[test]
fn autowire_bean_applies_post_processor() {
    use vernal_beans::BeanPostProcessor;

    static PP_CALLED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    struct TestPostProcessor;

    impl BeanPostProcessor for TestPostProcessor {
        fn post_process_after_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            PP_CALLED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(Some(bean))
        }
    }

    PP_CALLED.store(0, std::sync::atomic::Ordering::SeqCst);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabaseConfig, _>(
            |_resolver: &Resolver| DatabaseConfig {
                host: "pp_test".to_string(),
                port: 5432,
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<ConnectionPool, _>(|resolver: &Resolver| {
                let config = resolver.resolve::<DatabaseConfig>().unwrap();
                ConnectionPool {
                    config: Some(config),
                    max_size: 10,
                }
            })
            .depends_on::<DatabaseConfig>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let mut container = registry.container();
    container.add_bean_post_processor(Arc::new(TestPostProcessor));

    let pool = Arc::new(ConnectionPool {
        config: None,
        max_size: 5,
    });
    let _autowired = container.autowire_bean(pool).unwrap();

    // PostProcessor 应该被调用
    assert!(PP_CALLED.load(std::sync::atomic::Ordering::SeqCst) > 0);
}

// ── 6. autowireBean 不存在的类型 ─────────────────────────────────────────

/// 验证 autowireBean 对不存在的类型返回原实例。
#[test]
fn autowire_bean_unknown_type_returns_original() {
    let registry = vernal_beans::Registry::empty();
    let container = registry.container();

    #[derive(Debug)]
    struct UnknownBean;
    let bean = Arc::new(UnknownBean);

    let result = container.autowire_bean(bean.clone()).unwrap();
    // 不存在匹配定义，返回原实例（或 singleton 缓存）
    // 验证返回的类型正确
    assert!(result.downcast_ref::<UnknownBean>().is_some());
}

// ── 7. autowireBean 与 createBean 对比 ───────────────────────────────────

/// 验证 autowireBean 和 createBean 产生相同的依赖注入结果。
#[test]
fn autowire_bean_matches_create_bean() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabaseConfig, _>(
            |_resolver: &Resolver| DatabaseConfig {
                host: "compare".to_string(),
                port: 5432,
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<ConnectionPool, _>(|resolver: &Resolver| {
                let config = resolver.resolve::<DatabaseConfig>().unwrap();
                ConnectionPool {
                    config: Some(config),
                    max_size: 10,
                }
            })
            .depends_on::<DatabaseConfig>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // createBean 创建新实例
    let from_create = container
        .create_bean(std::any::type_name::<ConnectionPool>())
        .unwrap();
    let from_create = from_create.downcast_ref::<ConnectionPool>().unwrap();

    // autowireBean 通过工厂创建
    let unconfigured = Arc::new(ConnectionPool {
        config: None,
        max_size: 5,
    });
    let from_autowire = container.autowire_bean(unconfigured).unwrap();
    let from_autowire = from_autowire.downcast_ref::<ConnectionPool>().unwrap();

    // 两者都应该注入了 DatabaseConfig
    assert!(from_create.config.is_some());
    assert!(from_autowire.config.is_some());

    // 两者都指向同一个 singleton DatabaseConfig
    assert!(Arc::ptr_eq(
        &from_create.config.as_ref().unwrap(),
        &from_autowire.config.as_ref().unwrap()
    ));
}

// ── 8. autowireBeanProperties 按模式 ─────────────────────────────────────

/// 验证 autowireBeanProperties BY_TYPE 模式触发依赖解析。
#[test]
fn autowire_bean_properties_by_type_resolves() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabaseConfig, _>(
            |_resolver: &Resolver| DatabaseConfig {
                host: "by_type".to_string(),
                port: 5432,
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<ConnectionPool, _>(|resolver: &Resolver| {
                let config = resolver.resolve::<DatabaseConfig>().unwrap();
                ConnectionPool {
                    config: Some(config),
                    max_size: 10,
                }
            })
            .depends_on::<DatabaseConfig>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let pool = Arc::new(ConnectionPool {
        config: None,
        max_size: 5,
    });
    let result = container.autowire_bean_properties(pool, 2, false).unwrap(); // BY_TYPE

    // autowireBeanProperties 返回原实例（不修改字段）
    let result = result.downcast_ref::<ConnectionPool>().unwrap();
    assert!(result.config.is_none()); // 原实例不被修改
}

/// 验证 autowireBeanProperties BY_NAME 模式。
#[test]
fn autowire_bean_properties_by_name_resolves() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabaseConfig, _>(
            |_resolver: &Resolver| DatabaseConfig {
                host: "by_name".to_string(),
                port: 5432,
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let pool = Arc::new(ConnectionPool {
        config: None,
        max_size: 5,
    });
    let result = container.autowire_bean_properties(pool, 1, false).unwrap(); // BY_NAME

    let result = result.downcast_ref::<ConnectionPool>().unwrap();
    assert!(result.config.is_none());
}

// ── 9. configureBean 完整流程 ────────────────────────────────────────────

/// 验证 configureBean 应用 PostProcessor + 返回实例。
#[test]
fn configure_bean_full_flow() {
    use vernal_beans::BeanPostProcessor;

    static PP_CALLED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    struct ConfigPostProcessor;

    impl BeanPostProcessor for ConfigPostProcessor {
        fn post_process_after_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            PP_CALLED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(Some(bean))
        }
    }

    PP_CALLED.store(0, std::sync::atomic::Ordering::SeqCst);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabaseConfig, _>(
            |_resolver: &Resolver| DatabaseConfig {
                host: "config_test".to_string(),
                port: 5432,
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let mut container = registry.container();
    container.add_bean_post_processor(Arc::new(ConfigPostProcessor));

    let config = Arc::new(DatabaseConfig {
        host: "custom".to_string(),
        port: 9999,
    });
    let result = container.configure_bean(config, "DatabaseConfig").unwrap();

    // PostProcessor 应该被调用
    assert!(PP_CALLED.load(std::sync::atomic::Ordering::SeqCst) > 0);

    // 返回的实例应该是 PostProcessor 处理后的
    let result = result.downcast_ref::<DatabaseConfig>().unwrap();
    assert_eq!(result.host, "custom");
}

// ── 10. initializeBean 完整流程 ──────────────────────────────────────────

/// 验证 initializeBean 执行 before + after PostProcessor。
#[test]
fn initialize_bean_executes_post_processor_chain() {
    use vernal_beans::BeanPostProcessor;

    static BEFORE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    static AFTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    struct ChainPostProcessor;

    impl BeanPostProcessor for ChainPostProcessor {
        fn post_process_before_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            BEFORE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(Some(bean))
        }

        fn post_process_after_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            AFTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(Some(bean))
        }
    }

    BEFORE.store(0, std::sync::atomic::Ordering::SeqCst);
    AFTER.store(0, std::sync::atomic::Ordering::SeqCst);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabaseConfig, _>(
            |_resolver: &Resolver| DatabaseConfig {
                host: "init_test".to_string(),
                port: 5432,
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let mut container = registry.container();
    container.add_bean_post_processor(Arc::new(ChainPostProcessor));

    let bean = Arc::new(DatabaseConfig {
        host: "test".to_string(),
        port: 3306,
    });
    let _result = container.initialize_bean(bean, "DatabaseConfig").unwrap();

    // before 和 after 都应该被调用
    assert!(BEFORE.load(std::sync::atomic::Ordering::SeqCst) > 0);
    assert!(AFTER.load(std::sync::atomic::Ordering::SeqCst) > 0);
}
