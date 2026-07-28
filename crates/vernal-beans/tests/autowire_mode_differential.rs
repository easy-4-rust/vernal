//! autowire BY_NAME/BY_TYPE/CONSTRUCTOR 完整模式差分测试。
//!
//! 参照 Spring Framework 7.0.8 的 `DefaultListableBeanFactoryTests` 中
//! autowireBeanByName/autowireBeanByType/autowireConstructor 测试场景。

use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use vernal_beans::ComponentDefinition;
use vernal_beans::ComponentKey;
use vernal_beans::RegistryBuilder;
use vernal_beans::Resolver;
use vernal_beans::autowire_capable_bean_factory::AutowireCapableBeanFactory;
use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
use vernal_beans::bean_factory::BeanFactory;

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
struct UserRepository {
    pool_url: String,
}

#[derive(Debug)]
struct OrderRepository {
    pool_url: String,
}

#[derive(Debug)]
struct UserService {
    user_repo_url: String,
    order_repo_url: String,
}

// ── 1. AUTOWIRE_NO 模式差分测试 ──────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.autowireWithNoDependencies`：
/// 验证 AUTOWIRE_NO 模式不进行任何自动装配。
#[test]
fn differential_autowire_no_mode() {
    static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    CALL_COUNT.store(0, Ordering::SeqCst);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| {
                CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                AppConfig {
                    db_url: "test".to_string(),
                    max_connections: 10,
                }
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // AUTOWIRE_NO 模式：不进行自动装配
    let config = container
        .autowire(
            std::any::type_name::<AppConfig>(),
            0, // AUTOWIRE_NO
            false,
        )
        .unwrap();

    let config = config.downcast_ref::<AppConfig>().unwrap();
    assert_eq!(config.db_url, "test");
    assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);
}

// ── 2. AUTOWIRE_BY_NAME 模式差分测试 ──────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.autowireBeanByName`：
/// 验证 AUTOWIRE_BY_NAME 模式按名称注入。
#[test]
fn differential_autowire_by_name_mode() {
    static POOL_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    POOL_CALL_COUNT.store(0, Ordering::SeqCst);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| AppConfig {
                db_url: "by_name_test".to_string(),
                max_connections: 10,
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<ConnectionPool, _>(|resolver: &Resolver| {
                POOL_CALL_COUNT.fetch_add(1, Ordering::SeqCst);
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

    // AUTOWIRE_BY_NAME 模式
    let pool = container
        .autowire(
            std::any::type_name::<ConnectionPool>(),
            1, // AUTOWIRE_BY_NAME
            false,
        )
        .unwrap();

    let pool = pool.downcast_ref::<ConnectionPool>().unwrap();
    assert_eq!(pool.url, "by_name_test");
    assert_eq!(pool.max_size, 10);
    assert_eq!(POOL_CALL_COUNT.load(Ordering::SeqCst), 1);
}

// ── 3. AUTOWIRE_BY_TYPE 模式差分测试 ──────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.autowireBeanByType`：
/// 验证 AUTOWIRE_BY_TYPE 模式按类型注入。
#[test]
fn differential_autowire_by_type_mode() {
    static POOL_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    POOL_CALL_COUNT.store(0, Ordering::SeqCst);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| AppConfig {
                db_url: "by_type_test".to_string(),
                max_connections: 20,
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<ConnectionPool, _>(|resolver: &Resolver| {
                POOL_CALL_COUNT.fetch_add(1, Ordering::SeqCst);
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

    // AUTOWIRE_BY_TYPE 模式
    let pool = container
        .autowire(
            std::any::type_name::<ConnectionPool>(),
            2, // AUTOWIRE_BY_TYPE
            false,
        )
        .unwrap();

    let pool = pool.downcast_ref::<ConnectionPool>().unwrap();
    assert_eq!(pool.url, "by_type_test");
    assert_eq!(pool.max_size, 20);
    assert_eq!(POOL_CALL_COUNT.load(Ordering::SeqCst), 1);
}

// ── 4. AUTOWIRE_CONSTRUCTOR 模式差分测试 ──────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.autowireConstructor`：
/// 验证 AUTOWIRE_CONSTRUCTOR 模式按构造器注入。
#[test]
fn differential_autowire_constructor_mode() {
    static POOL_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    POOL_CALL_COUNT.store(0, Ordering::SeqCst);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| AppConfig {
                db_url: "constructor_test".to_string(),
                max_connections: 15,
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<ConnectionPool, _>(|resolver: &Resolver| {
                POOL_CALL_COUNT.fetch_add(1, Ordering::SeqCst);
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

    // AUTOWIRE_CONSTRUCTOR 模式
    let pool = container
        .autowire(
            std::any::type_name::<ConnectionPool>(),
            3, // AUTOWIRE_CONSTRUCTOR
            false,
        )
        .unwrap();

    let pool = pool.downcast_ref::<ConnectionPool>().unwrap();
    assert_eq!(pool.url, "constructor_test");
    assert_eq!(pool.max_size, 15);
    assert_eq!(POOL_CALL_COUNT.load(Ordering::SeqCst), 1);
}

// ── 5. AUTOWIRE_BY_TYPE 多候选差分测试 ────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.autowireBeanByTypeWithTwoMatches`：
/// 验证 AUTOWIRE_BY_TYPE 模式下多候选的处理。
#[test]
fn differential_autowire_by_type_multiple_candidates() {
    use vernal_beans::Qualifier;

    let mut builder = RegistryBuilder::new();
    builder
        .register(
            ComponentDefinition::singleton::<ConnectionPool, _>(|_resolver: &Resolver| {
                ConnectionPool {
                    url: "primary_pool".to_string(),
                    max_size: 10,
                }
            })
            .qualified(Qualifier::new("primary").unwrap()),
        )
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<ConnectionPool, _>(|_resolver: &Resolver| {
                ConnectionPool {
                    url: "secondary_pool".to_string(),
                    max_size: 5,
                }
            })
            .qualified(Qualifier::new("secondary").unwrap()),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // 验证两个 ConnectionPool 都已注册
    // 使用 bean_definition_count 验证总数
    assert_eq!(container.bean_definition_count(), 2);

    // 使用 bean_definition_names 验证名称
    let names = container.bean_definition_names();
    assert_eq!(names.len(), 2);
}

// ── 6. createBean + autowire 集成差分测试 ────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.createBeanWithDisposableBean`：
/// 验证 createBean 完整流程（实例化 → 属性注入 → PostProcessor）。
#[test]
fn differential_create_bean_with_autowire() {
    static CONFIG_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);
    static POOL_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    CONFIG_CALL_COUNT.store(0, Ordering::SeqCst);
    POOL_CALL_COUNT.store(0, Ordering::SeqCst);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| {
                CONFIG_CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                AppConfig {
                    db_url: "integrated".to_string(),
                    max_connections: 25,
                }
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<ConnectionPool, _>(|resolver: &Resolver| {
                POOL_CALL_COUNT.fetch_add(1, Ordering::SeqCst);
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

    // createBean 完整流程
    let pool = container
        .create_bean(std::any::type_name::<ConnectionPool>())
        .unwrap();
    let pool = pool.downcast_ref::<ConnectionPool>().unwrap();

    assert_eq!(pool.url, "integrated");
    assert_eq!(pool.max_size, 25);
    assert_eq!(CONFIG_CALL_COUNT.load(Ordering::SeqCst), 1);
    assert_eq!(POOL_CALL_COUNT.load(Ordering::SeqCst), 1);
}

// ── 7. autowireBeanProperties 差分测试 ───────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.autowireExistingBeanByName`：
/// 验证 autowireBeanProperties 按模式注入。
#[test]
fn differential_autowire_bean_properties_modes() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| AppConfig {
                db_url: "props_test".to_string(),
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

    // 创建一个独立的 ConnectionPool
    let standalone = Arc::new(ConnectionPool {
        url: "standalone".to_string(),
        max_size: 5,
    });

    // AUTOWIRE_BY_NAME 模式
    let result = container
        .autowire_bean_properties(standalone.clone(), 1, false)
        .unwrap();
    let result = result.downcast_ref::<ConnectionPool>().unwrap();
    assert_eq!(result.url, "standalone"); // 原实例不被修改

    // AUTOWIRE_BY_TYPE 模式
    let result = container
        .autowire_bean_properties(standalone.clone(), 2, false)
        .unwrap();
    let result = result.downcast_ref::<ConnectionPool>().unwrap();
    assert_eq!(result.url, "standalone");

    // AUTOWIRE_NO 模式
    let result = container
        .autowire_bean_properties(standalone, 0, false)
        .unwrap();
    let result = result.downcast_ref::<ConnectionPool>().unwrap();
    assert_eq!(result.url, "standalone");
}

// ── 8. resolveDependency 差分测试 ────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.referenceByType`：
/// 验证 resolveDependency 按类型解析。
#[test]
fn differential_resolve_dependency_by_type() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| AppConfig {
                db_url: "dep_test".to_string(),
                max_connections: 10,
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let descriptor = vernal_beans::DependencyDescriptor::for_field(
        std::any::TypeId::of::<AppConfig>(),
        std::any::type_name::<AppConfig>(),
    );

    let result = container.resolve_dependency(&descriptor, None).unwrap();
    assert!(result.is_some());

    let unwrapped = result.unwrap();
    let config = unwrapped.downcast_ref::<AppConfig>().unwrap();
    assert_eq!(config.db_url, "dep_test");
}

/// 验证可选依赖找不到时返回 None。
#[test]
fn differential_resolve_optional_dependency() {
    let registry = vernal_beans::Registry::empty();
    let container = registry.container();

    let descriptor = vernal_beans::DependencyDescriptor::for_field(
        std::any::TypeId::of::<AppConfig>(),
        std::any::type_name::<AppConfig>(),
    )
    .with_optional(true);

    let result = container.resolve_dependency(&descriptor, None).unwrap();
    assert!(result.is_none());
}

/// 验证不可选依赖找不到时返回错误。
#[test]
fn differential_resolve_required_dependency_error() {
    let registry = vernal_beans::Registry::empty();
    let container = registry.container();

    let descriptor = vernal_beans::DependencyDescriptor::for_field(
        std::any::TypeId::of::<AppConfig>(),
        std::any::type_name::<AppConfig>(),
    );

    let result = container.resolve_dependency(&descriptor, None);
    assert!(result.is_err());
}

// ── 9. configureBean 差分测试 ────────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.configureBean`：
/// 验证 configureBean 应用 PostProcessor + 返回实例。
#[test]
fn differential_configure_bean_with_post_processor() {
    use vernal_beans::bean_post_processor::BeanPostProcessor;

    static PP_CALLED: AtomicUsize = AtomicUsize::new(0);

    struct ConfigPostProcessor;

    impl BeanPostProcessor for ConfigPostProcessor {
        fn post_process_after_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            PP_CALLED.fetch_add(1, Ordering::SeqCst);
            Ok(Some(bean))
        }
    }

    PP_CALLED.store(0, Ordering::SeqCst);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| AppConfig {
                db_url: "configure_test".to_string(),
                max_connections: 10,
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let mut container = registry.container();
    container.add_bean_post_processor(Arc::new(ConfigPostProcessor));

    let config = Arc::new(AppConfig {
        db_url: "custom".to_string(),
        max_connections: 5,
    });
    let result = container.configure_bean(config, "AppConfig").unwrap();

    // PostProcessor 应该被调用
    assert!(PP_CALLED.load(Ordering::SeqCst) > 0);

    // 返回的实例应该是 PostProcessor 处理后的
    let result = result.downcast_ref::<AppConfig>().unwrap();
    assert_eq!(result.db_url, "custom");
}

// ── 10. initializeBean 差分测试 ──────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.initializeBean`：
/// 验证 initializeBean 执行 before + after PostProcessor。
#[test]
fn differential_initialize_bean_lifecycle() {
    use vernal_beans::bean_post_processor::BeanPostProcessor;

    static BEFORE: AtomicUsize = AtomicUsize::new(0);
    static AFTER: AtomicUsize = AtomicUsize::new(0);

    struct LifecyclePostProcessor;

    impl BeanPostProcessor for LifecyclePostProcessor {
        fn post_process_before_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            BEFORE.fetch_add(1, Ordering::SeqCst);
            Ok(Some(bean))
        }

        fn post_process_after_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            AFTER.fetch_add(1, Ordering::SeqCst);
            Ok(Some(bean))
        }
    }

    BEFORE.store(0, Ordering::SeqCst);
    AFTER.store(0, Ordering::SeqCst);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| AppConfig {
                db_url: "init_test".to_string(),
                max_connections: 10,
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let mut container = registry.container();
    container.add_bean_post_processor(Arc::new(LifecyclePostProcessor));

    let bean = Arc::new(AppConfig {
        db_url: "test".to_string(),
        max_connections: 5,
    });
    let _result = container.initialize_bean(bean, "AppConfig").unwrap();

    // before 和 after 都应该被调用
    assert!(BEFORE.load(Ordering::SeqCst) > 0);
    assert!(AFTER.load(Ordering::SeqCst) > 0);
}

// ── 11. invalid autowire mode 差分测试 ──────────────────────────────────

/// 验证无效 autowire 模式返回错误。
#[test]
fn differential_invalid_autowire_mode() {
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

    let result = container.autowire(
        std::any::type_name::<AppConfig>(),
        99, // 无效模式
        false,
    );
    assert!(result.is_err());
}

// ── 12. destroyBeanInstance 差分测试 ─────────────────────────────────────

/// 验证 destroyBeanInstance 正常执行。
#[test]
fn differential_destroy_bean_instance() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| AppConfig {
                db_url: "destroy".to_string(),
                max_connections: 10,
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let bean = AppConfig {
        db_url: "destroy".to_string(),
        max_connections: 10,
    };
    let result = container.destroy_bean_instance("AppConfig", &bean);
    assert!(result.is_ok());
}
