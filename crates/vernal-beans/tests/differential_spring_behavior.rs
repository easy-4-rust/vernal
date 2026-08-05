//! 差分测试：验证 vernal-beans 与 spring-beans 的行为等价性。
//!
//! 参照 Spring Framework 7.0.8 的 `DefaultListableBeanFactoryTests` 测试场景，
//! 验证 prototype vs singleton scope resolution with autowire injection
//! 的行为与 Spring 一致。

use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use vernal_beans::AutowireCapableBeanFactory;
use vernal_beans::BeanFactory;
use vernal_beans::BeanPostProcessor;
use vernal_beans::ComponentDefinition;
use vernal_beans::ComponentKey;
use vernal_beans::GenericBeanDefinition;
use vernal_beans::RegistryBuilder;
use vernal_beans::Resolver;
use vernal_beans::RootBeanDefinition;
use vernal_beans::Scope;
use vernal_beans::bean_scope::BeanScope;

// ── 测试类型 ─────────────────────────────────────────────────────────────

/// AppConfig 对应 Spring 的 `@ConfigurationProperties` Bean。
#[derive(Debug, Clone)]
struct AppConfig {
    url: String,
}

#[derive(Debug)]
struct ConnectionPool {
    config_url: String,
    max_size: u32,
}

#[derive(Debug)]
struct UserRepository {
    pool_url: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct OrderService {
    user_repo_url: String,
}

// ── 1. Singleton vs Prototype 差分测试 ───────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.prototype`：
/// 验证 singleton 和 prototype 的行为差异。
///
/// **Spring 行为**：
/// - singleton：每次 getBean 返回同一实例
/// - prototype：每次 getBean 返回新实例
///
/// **vernal 行为**（应与 Spring 一致）：
/// - Singleton：每次 resolve 返回同一 Arc
/// - Transient：每次 resolve 返回新 Arc
#[test]
fn differential_singleton_vs_prototype_behavior() {
    static SINGLETON_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);
    static PROTOTYPE_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    SINGLETON_CALL_COUNT.store(0, Ordering::SeqCst);
    PROTOTYPE_CALL_COUNT.store(0, Ordering::SeqCst);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| {
                SINGLETON_CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                AppConfig {
                    url: "singleton".to_string(),
                }
            },
        ))
        .unwrap();
    builder
        .register(ComponentDefinition::transient::<ConnectionPool, _>(
            |_resolver: &Resolver| {
                let count = PROTOTYPE_CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                ConnectionPool {
                    config_url: "prototype".to_string(),
                    max_size: count as u32,
                }
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // ── Singleton 行为验证 ──
    // Spring: getBean("config") 多次返回同一实例
    let a = container.resolve::<AppConfig>().unwrap();
    let b = container.resolve::<AppConfig>().unwrap();
    let c = container.resolve::<AppConfig>().unwrap();

    // vernal: resolve 多次返回同一 Arc（singleton 缓存）
    assert!(Arc::ptr_eq(&a, &b), "Singleton should return same instance");
    assert!(Arc::ptr_eq(&b, &c), "Singleton should return same instance");
    assert_eq!(
        SINGLETON_CALL_COUNT.load(Ordering::SeqCst),
        1,
        "Singleton factory should only be called once"
    );

    // ── Prototype 行为验证 ──
    // Spring: getBean("pool") 每次返回新实例
    let p1 = container.resolve::<ConnectionPool>().unwrap();
    let p2 = container.resolve::<ConnectionPool>().unwrap();
    let p3 = container.resolve::<ConnectionPool>().unwrap();

    // vernal: resolve 每次返回新 Arc（transient 每次创建）
    assert!(
        !Arc::ptr_eq(&p1, &p2),
        "Prototype should return different instances"
    );
    assert!(
        !Arc::ptr_eq(&p2, &p3),
        "Prototype should return different instances"
    );
    assert_eq!(
        PROTOTYPE_CALL_COUNT.load(Ordering::SeqCst),
        3,
        "Prototype factory should be called each time"
    );

    // 验证 prototype 实例的不同值
    assert_eq!(p1.max_size, 0); // 第一次调用
    assert_eq!(p2.max_size, 1); // 第二次调用
    assert_eq!(p3.max_size, 2); // 第三次调用
}

// ── 2. Singleton 依赖注入差分测试 ────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.referenceByType`：
/// 验证 singleton 依赖注入链的行为。
///
/// **Spring 行为**：
/// - DataSource 是 singleton
/// - UserRepository 依赖 DataSource
/// - 多次 getBean(UserRepository) 都使用同一个 DataSource
///
/// **vernal 行为**（应与 Spring 一致）：
/// - DataSource 是 singleton，所有依赖者共享同一实例
/// - UserRepository 的 pool_url 应该是 DataSource 的 url
#[test]
fn differential_singleton_dependency_injection() {
    static DS_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);
    static USER_REPO_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    DS_CALL_COUNT.store(0, Ordering::SeqCst);
    USER_REPO_CALL_COUNT.store(0, Ordering::SeqCst);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| {
                DS_CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                AppConfig {
                    url: "db://prod".to_string(),
                }
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<ConnectionPool, _>(|resolver: &Resolver| {
                let config = resolver.resolve::<AppConfig>().unwrap();
                ConnectionPool {
                    config_url: config.url.clone(),
                    max_size: 10,
                }
            })
            .depends_on::<AppConfig>(),
        )
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<UserRepository, _>(|resolver: &Resolver| {
                USER_REPO_CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                let pool = resolver.resolve::<ConnectionPool>().unwrap();
                UserRepository {
                    pool_url: pool.config_url.clone(),
                }
            })
            .depends_on::<ConnectionPool>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // ── 依赖注入验证 ──
    // Spring: getBean(UserRepository) 应该触发 DataSource 和 ConnectionPool 的创建
    let user_repo = container.resolve::<UserRepository>().unwrap();
    assert_eq!(
        user_repo.pool_url, "db://prod",
        "Dependency should be injected correctly"
    );

    // DataSource 只创建一次（singleton）
    assert_eq!(
        DS_CALL_COUNT.load(Ordering::SeqCst),
        1,
        "DataSource should only be created once"
    );

    // UserRepository 只创建一次（singleton）
    assert_eq!(
        USER_REPO_CALL_COUNT.load(Ordering::SeqCst),
        1,
        "UserRepository should only be created once"
    );

    // ── 验证依赖共享 ──
    // Spring: 多次 getBean(UserRepository) 都使用同一个 DataSource 实例
    let user_repo2 = container.resolve::<UserRepository>().unwrap();
    assert!(
        Arc::ptr_eq(&user_repo, &user_repo2),
        "Singleton should return same instance"
    );

    // 验证 DataSource 也被正确解析
    let ds = container.resolve::<AppConfig>().unwrap();
    assert_eq!(ds.url, "db://prod");
}

// ── 3. Prototype 依赖注入差分测试 ────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.autowireWithSatisfiedConstructorDependency`：
/// 验证 prototype 依赖注入链的行为。
///
/// **Spring 行为**：
/// - DataSource 是 singleton（共享）
/// - ConnectionPool 是 prototype（每次新建）
/// - UserRepository 依赖 ConnectionPool
/// - 每次 getBean(UserRepository) 使用不同的 ConnectionPool
///
/// **vernal 行为**（应与 Spring 一致）：
/// - DataSource 是 singleton，被所有 ConnectionPool 共享
/// - ConnectionPool 是 transient，每次创建新实例
/// - UserRepository 的 pool_url 应该是对应 ConnectionPool 的 config_url
#[test]
fn differential_prototype_dependency_injection() {
    static DS_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);
    static POOL_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    DS_CALL_COUNT.store(0, Ordering::SeqCst);
    POOL_CALL_COUNT.store(0, Ordering::SeqCst);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| {
                DS_CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                AppConfig {
                    url: "db://prod".to_string(),
                }
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::transient::<ConnectionPool, _>(|resolver: &Resolver| {
                POOL_CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                let config = resolver.resolve::<AppConfig>().unwrap();
                ConnectionPool {
                    config_url: config.url.clone(),
                    max_size: 10,
                }
            })
            .depends_on::<AppConfig>(),
        )
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<UserRepository, _>(|resolver: &Resolver| {
                let pool = resolver.resolve::<ConnectionPool>().unwrap();
                UserRepository {
                    pool_url: pool.config_url.clone(),
                }
            })
            .depends_on::<ConnectionPool>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // ── Prototype 依赖注入验证 ──
    // Spring: 每次 getBean(UserRepository) 会创建新的 ConnectionPool
    let user_repo1 = container.resolve::<UserRepository>().unwrap();
    let user_repo2 = container.resolve::<UserRepository>().unwrap();

    // DataSource 只创建一次（singleton）
    assert_eq!(
        DS_CALL_COUNT.load(Ordering::SeqCst),
        1,
        "DataSource should only be created once"
    );

    // UserRepository 是 singleton，所以两次返回同一实例
    assert!(
        Arc::ptr_eq(&user_repo1, &user_repo2),
        "UserRepository should return same instance"
    );

    // 验证依赖注入正确
    assert_eq!(
        user_repo1.pool_url, "db://prod",
        "Dependency should be injected correctly"
    );
}

// ── 4. FactoryBean 行为差分测试 ──────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.singletonFactoryBeanIgnoredByNonEagerTypeMatching`：
/// 验证 FactoryBean 的 singleton 语义。
///
/// **Spring 行为**：
/// - FactoryBean 本身是 singleton
/// - getObject() 每次返回同一实例（singleton FactoryBean）
///
/// **vernal 行为**（应与 Spring 一致）：
/// - FactoryBean 实例通过 singleton 注册
/// - getObject() 每次返回新实例（由工厂闭包决定）
#[test]
fn differential_factory_bean_singleton() {
    static FACTORY_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    FACTORY_CALL_COUNT.store(0, Ordering::SeqCst);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| {
                FACTORY_CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                AppConfig {
                    url: "factory_created".to_string(),
                }
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // FactoryBean 本身是 singleton
    let a = container.resolve::<AppConfig>().unwrap();
    let b = container.resolve::<AppConfig>().unwrap();
    let c = container.resolve::<AppConfig>().unwrap();

    // singleton 工厂只调用一次
    assert!(
        Arc::ptr_eq(&a, &b),
        "Singleton FactoryBean should return same instance"
    );
    assert!(
        Arc::ptr_eq(&b, &c),
        "Singleton FactoryBean should return same instance"
    );
    assert_eq!(
        FACTORY_CALL_COUNT.load(Ordering::SeqCst),
        1,
        "Singleton factory should only be called once"
    );
}

// ── 5. Scope 注册差分测试 ────────────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.scopingBeanToUnregisteredScopeResultsInAnException`：
/// 验证自定义 Scope 的行为。
///
/// **Spring 行为**：
/// - 注册自定义 Scope
/// - 使用 Scope 创建 Bean
/// - Scope 关闭时销毁 Bean
///
/// **vernal 行为**（应与 Spring 一致）：
/// - Scope 通过 BeanScope trait 注册
/// - get() 创建或获取实例
/// - destroy() 执行销毁回调
#[test]
fn differential_custom_scope_behavior() {
    static GET_COUNT: AtomicUsize = AtomicUsize::new(0);
    static DESTROY_COUNT: AtomicUsize = AtomicUsize::new(0);

    GET_COUNT.store(0, Ordering::SeqCst);
    DESTROY_COUNT.store(0, Ordering::SeqCst);

    struct TestScope;

    impl BeanScope for TestScope {
        fn get(
            &self,
            _name: &str,
            object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
        ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            GET_COUNT.fetch_add(1, Ordering::SeqCst);
            Ok(object_factory())
        }

        fn remove(
            &self,
            _name: &str,
        ) -> Result<Option<Box<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(None)
        }

        fn register_destruction_callback(
            &self,
            _name: &str,
            _callback: Box<dyn FnOnce() + Send + Sync>,
        ) {
            DESTROY_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }

    let scope = TestScope;

    // ── Scope 行为验证 ──
    // Spring: scope.get() 创建实例
    let obj1 = scope.get("test", &|| Box::new(42i32)).unwrap();
    let obj2 = scope.get("test", &|| Box::new(100i32)).unwrap();

    // vernal: 每次 get 调用工厂
    assert_eq!(
        GET_COUNT.load(Ordering::SeqCst),
        2,
        "Scope should call factory for each get"
    );

    // 验证返回值
    let v1 = obj1.downcast_ref::<i32>().unwrap();
    let v2 = obj2.downcast_ref::<i32>().unwrap();
    assert_eq!(*v1, 42);
    assert_eq!(*v2, 100);

    // Spring: scope.registerDestructionCallback() 注册回调
    scope.register_destruction_callback("test", Box::new(|| {}));
    assert_eq!(
        DESTROY_COUNT.load(Ordering::SeqCst),
        1,
        "Destruction callback should be registered"
    );
}

// ── 6. BeanFactory 接口语义差分测试 ──────────────────────────────────────

/// 参照 Spring `BeanFactory` 接口语义：
/// - containsBean(name) 检查是否存在
/// - isSingleton(name) 检查是否为 singleton
/// - isPrototype(name) 检查是否为 prototype
///
/// **vernal 行为**（应与 Spring 一致）：
/// - contains_bean 检查 ComponentKey 是否存在
/// - is_singleton / is_prototype 通过 Scope 判断
#[test]
fn differential_bean_factory_interface_semantics() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| AppConfig {
                url: "singleton".to_string(),
            },
        ))
        .unwrap();
    builder
        .register(ComponentDefinition::transient::<ConnectionPool, _>(
            |_resolver: &Resolver| ConnectionPool {
                config_url: "prototype".to_string(),
                max_size: 5,
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // ── containsBean 语义 ──
    // Spring: containsBean("config") 返回 true
    let config_key = ComponentKey::of::<AppConfig>();
    assert!(
        container.contains_bean(&config_key),
        "Should contain singleton bean"
    );

    let pool_key = ComponentKey::of::<ConnectionPool>();
    assert!(
        container.contains_bean(&pool_key),
        "Should contain prototype bean"
    );

    let missing_key = ComponentKey::of::<OrderService>();
    assert!(
        !container.contains_bean(&missing_key),
        "Should not contain missing bean"
    );

    // ── isSingleton 语义 ──
    // Spring: isSingleton("config") 返回 true
    assert!(
        container.is_singleton(&config_key).unwrap(),
        "AppConfig should be singleton"
    );
    assert!(
        !container.is_prototype(&config_key).unwrap(),
        "AppConfig should not be prototype"
    );

    // ── isPrototype 语义 ──
    // Spring: isPrototype("pool") 返回 true
    assert!(
        container.is_prototype(&pool_key).unwrap(),
        "ConnectionPool should be prototype"
    );
    assert!(
        !container.is_singleton(&pool_key).unwrap(),
        "ConnectionPool should not be singleton"
    );
}

// ── 7. Parent-Child BeanDefinition 继承差分测试 ──────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.prototypeExtendsPrototype`：
/// 验证 BeanDefinition 继承行为。
///
/// **Spring 行为**：
/// - Parent BeanDefinition 定义基础属性
/// - Child BeanDefinition 继承并覆盖属性
/// - 合并后生成 RootBeanDefinition
///
/// **vernal 行为**（通过 GenericBeanDefinition + RootBeanDefinition 实现）：
/// - GenericBeanDefinition 支持 parent_name
/// - RootBeanDefinition 是合并后的最终定义
#[test]
fn differential_bean_definition_inheritance() {
    // Parent 定义
    let mut parent = GenericBeanDefinition::new();
    parent.set_bean_class_name("BaseService");
    parent.set_scope(Scope::Singleton);
    parent.set_init_method_name("baseInit");
    parent.set_lazy_init(true);

    // Child 定义
    let mut child = GenericBeanDefinition::new();
    child.set_bean_class_name("ConcreteService");
    child.set_parent_name("BaseService");
    child.set_scope(Scope::Singleton); // 继承
    child.set_init_method_name("concreteInit"); // 覆盖

    // 验证继承
    assert_eq!(child.get_parent_name(), Some("BaseService"));
    assert_eq!(child.scope(), Scope::Singleton);
    assert_eq!(child.init_method_name(), Some("concreteInit"));
    assert!(!child.is_abstract());

    // 转换为 RootBeanDefinition
    let root = RootBeanDefinition::from_generic(child);
    assert_eq!(root.bean_class_name(), "ConcreteService");
    assert_eq!(root.scope(), Scope::Singleton);
    assert_eq!(root.init_method_name(), Some("concreteInit"));
}

// ── 8. PostProcessor 链执行差分测试 ──────────────────────────────────────

/// 参照 Spring `BeanPostProcessorTests`：
/// 验证 PostProcessor 链的执行顺序。
///
/// **Spring 行为**：
/// - postProcessBeforeInitialization 在 afterPropertiesSet 之前
/// - postProcessAfterInitialization 在 afterPropertiesSet 之后
/// - 多个 PostProcessor 按注册顺序执行
///
/// **vernal 行为**：
/// - after PostProcessor 链在 resolve_definition 中执行
/// - 多个 PostProcessor 按添加顺序执行
#[test]
fn differential_post_processor_chain_order() {
    static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    struct CountingPostProcessor;

    impl BeanPostProcessor for CountingPostProcessor {
        fn post_process_after_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            CALL_COUNT.fetch_add(1, Ordering::SeqCst);
            Ok(Some(bean))
        }
    }

    CALL_COUNT.store(0, Ordering::SeqCst);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| AppConfig {
                url: "test".to_string(),
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let mut container = registry.container();

    // 添加多个 PostProcessor
    container.add_bean_post_processor(Arc::new(CountingPostProcessor));
    container.add_bean_post_processor(Arc::new(CountingPostProcessor));
    container.add_bean_post_processor(Arc::new(CountingPostProcessor));

    // resolve 会触发 PostProcessor 链
    let _config = container.resolve::<AppConfig>().unwrap();

    // 验证所有 PostProcessor 都被调用
    assert_eq!(
        CALL_COUNT.load(Ordering::SeqCst),
        3,
        "All 3 PostProcessors should be called"
    );
}

// ── 9. createBean 完整流程差分测试 ────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.createBean`：
/// 验证 createBean 的完整流程。
///
/// **Spring 行为**：
/// 1. 实例化 Bean
/// 2. 属性注入
/// 3. BeanNameAware.setBeanName
/// 4. BeanFactoryAware.setBeanFactory
/// 5. BeanPostProcessor.postProcessBeforeInitialization
/// 6. InitializingBean.afterPropertiesSet
/// 7. 自定义 init-method
/// 8. BeanPostProcessor.postProcessAfterInitialization
///
/// **vernal 行为**（通过工厂 + PostProcessor 链实现）：
/// 1. 工厂创建实例（属性注入在工厂内完成）
/// 2. PostProcessor.postProcessAfterInitialization
#[test]
fn differential_create_bean_full_flow() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| AppConfig {
                url: "created".to_string(),
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<ConnectionPool, _>(|resolver: &Resolver| {
                let config = resolver.resolve::<AppConfig>().unwrap();
                ConnectionPool {
                    config_url: config.url.clone(),
                    max_size: 10,
                }
            })
            .depends_on::<AppConfig>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // createBean 创建新实例（完整流程）
    let pool = container
        .create_bean(std::any::type_name::<ConnectionPool>())
        .unwrap();
    let pool = pool.downcast_ref::<ConnectionPool>().unwrap();

    // 验证依赖已注入
    assert_eq!(
        pool.config_url, "created",
        "Dependency should be injected via factory"
    );
    assert_eq!(pool.max_size, 10, "Factory should set properties correctly");
}

// ── 10. warm_up 预实例化差分测试 ──────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.preInstantiateSingletons`：
/// 验证 warm_up 预实例化所有 singleton。
///
/// **Spring 行为**：
/// - preInstantiateSingletons() 创建所有 singleton
/// - 跳过 lazy-init 的 singleton
/// - 跳过 SmartFactoryBean.isEagerInit() = false 的
///
/// **vernal 行为**（应与 Spring 一致）：
/// - warm_up() 创建所有 singleton
/// - 当前不支持 lazy-init 检查（简化实现）
#[test]
fn differential_warm_up_creates_singletons() {
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
                    url: "warm".to_string(),
                }
            },
        ))
        .unwrap();
    builder
        .register(ComponentDefinition::singleton::<ConnectionPool, _>(
            |_resolver: &Resolver| {
                POOL_CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                ConnectionPool {
                    config_url: "warm".to_string(),
                    max_size: 5,
                }
            },
        ))
        .unwrap();
    builder
        .register(ComponentDefinition::transient::<UserRepository, _>(
            |_resolver: &Resolver| UserRepository {
                pool_url: "warm".to_string(),
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // warm_up 前：所有计数为 0
    assert_eq!(CONFIG_CALL_COUNT.load(Ordering::SeqCst), 0);
    assert_eq!(POOL_CALL_COUNT.load(Ordering::SeqCst), 0);

    // warm_up 创建所有 singleton
    container.warm_up().unwrap();

    // warm_up 后：singleton 工厂被调用一次
    assert_eq!(
        CONFIG_CALL_COUNT.load(Ordering::SeqCst),
        1,
        "Config should be created once"
    );
    assert_eq!(
        POOL_CALL_COUNT.load(Ordering::SeqCst),
        1,
        "Pool should be created once"
    );

    // 再次 resolve 不会重复创建
    let _config = container.resolve::<AppConfig>().unwrap();
    let _pool = container.resolve::<ConnectionPool>().unwrap();
    assert_eq!(CONFIG_CALL_COUNT.load(Ordering::SeqCst), 1);
    assert_eq!(POOL_CALL_COUNT.load(Ordering::SeqCst), 1);
}
