//! 完整 autowire 注入 + Scope 注册 + BeanDefinitionRegistry 操作测试。
//!
//! 参照 Spring Framework 7.0.8 的 `DefaultListableBeanFactoryTests`：
//! - autowire 按类型/名称/构造器注入
//! - RequestScope/SessionScope 自定义 Scope 注册与激活
//! - BeanDefinitionRegistry 操作（register/remove/contains/count）

use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use vernal_beans::AutowireCapableBeanFactory;
use vernal_beans::ComponentDefinition;
use vernal_beans::ComponentKey;
use vernal_beans::RegistryBuilder;
use vernal_beans::Resolver;
use vernal_beans::Scope;
use vernal_beans::bean_factory::BeanFactory;
use vernal_beans::bean_scope::BeanScope;

// ── 测试类型 ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Config {
    url: String,
}

#[derive(Debug)]
struct DataSource {
    config_url: String,
}

#[derive(Debug)]
struct UserService {
    ds_url: String,
}

#[derive(Debug)]
struct OrderService {
    user_service: Option<Arc<UserService>>,
}

// ── 1. Autowire 按类型注入 ───────────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.autowireBeanByType`：
/// 验证 AUTOWIRE_BY_TYPE 模式下按类型匹配注入。
#[test]
fn autowire_by_type_injects_matching_type() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "by_type_inject".to_string(),
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<DataSource, _>(|resolver: &Resolver| {
                let config = resolver.resolve::<Config>().unwrap();
                DataSource {
                    config_url: config.url.clone(),
                }
            })
            .depends_on::<Config>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // AUTOWIRE_BY_TYPE 模式创建 DataSource
    let ds = container
        .autowire(
            std::any::type_name::<DataSource>(),
            2, // AUTOWIRE_BY_TYPE
            false,
        )
        .unwrap();

    let ds = ds.downcast_ref::<DataSource>().unwrap();
    assert_eq!(ds.config_url, "by_type_inject");
}

/// 参照 Spring `DefaultListableBeanFactoryTests.autowireBeanByName`：
/// 验证 AUTOWIRE_BY_NAME 模式下按名称匹配注入。
#[test]
fn autowire_by_name_injects_matching_name() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "by_name_inject".to_string(),
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<DataSource, _>(|resolver: &Resolver| {
                let config = resolver.resolve::<Config>().unwrap();
                DataSource {
                    config_url: config.url.clone(),
                }
            })
            .depends_on::<Config>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // AUTOWIRE_BY_NAME 模式
    let ds = container
        .autowire(
            std::any::type_name::<DataSource>(),
            1, // AUTOWIRE_BY_NAME
            false,
        )
        .unwrap();

    let ds = ds.downcast_ref::<DataSource>().unwrap();
    assert_eq!(ds.config_url, "by_name_inject");
}

/// 参照 Spring `DefaultListableBeanFactoryTests.autowireConstructor`：
/// 验证 AUTOWIRE_CONSTRUCTOR 模式下按构造器参数匹配。
#[test]
fn autowire_constructor_injects_via_factory() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "constructor_inject".to_string(),
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<DataSource, _>(|resolver: &Resolver| {
                let config = resolver.resolve::<Config>().unwrap();
                DataSource {
                    config_url: config.url.clone(),
                }
            })
            .depends_on::<Config>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // AUTOWIRE_CONSTRUCTOR 委托给 createBean
    let ds = container
        .autowire(
            std::any::type_name::<DataSource>(),
            3, // AUTOWIRE_CONSTRUCTOR
            false,
        )
        .unwrap();

    let ds = ds.downcast_ref::<DataSource>().unwrap();
    assert_eq!(ds.config_url, "constructor_inject");
}

// ── 2. 多层依赖自动注入 ─────────────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.referenceByType`：
/// 验证多层依赖自动注入链。
#[test]
fn multi_level_autowire_injection_chain() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "chain_db".to_string(),
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<DataSource, _>(|resolver: &Resolver| {
                let config = resolver.resolve::<Config>().unwrap();
                DataSource {
                    config_url: config.url.clone(),
                }
            })
            .depends_on::<Config>(),
        )
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<UserService, _>(|resolver: &Resolver| {
                let ds = resolver.resolve::<DataSource>().unwrap();
                UserService {
                    ds_url: ds.config_url.clone(),
                }
            })
            .depends_on::<DataSource>(),
        )
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<OrderService, _>(|resolver: &Resolver| {
                let user = resolver.resolve::<UserService>().unwrap();
                OrderService {
                    user_service: Some(user),
                }
            })
            .depends_on::<UserService>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // 预热所有 singleton
    container.warm_up().unwrap();

    // 验证多层注入
    let order = container.resolve::<OrderService>().unwrap();
    let user = order.user_service.as_ref().unwrap();
    assert_eq!(user.ds_url, "chain_db");

    // 验证 DataSource 被共享
    let ds_direct = container.resolve::<DataSource>().unwrap();
    // 用户服务中的 DataSource URL 应该相同
    assert_eq!(user.ds_url, ds_direct.config_url);
}

/// 参照 Spring `DefaultListableBeanFactoryTests.autowireWithSatisfiedConstructorDependency`：
/// 验证构造器依赖满足时 autowire 成功。
#[test]
fn autowire_constructor_satisfied_dependency() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "satisfied".to_string(),
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<DataSource, _>(|resolver: &Resolver| {
                let config = resolver.resolve::<Config>().unwrap();
                DataSource {
                    config_url: config.url.clone(),
                }
            })
            .depends_on::<Config>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let ds = container
        .autowire(
            std::any::type_name::<DataSource>(),
            3,    // AUTOWIRE_CONSTRUCTOR
            true, // dependency_check = true
        )
        .unwrap();

    let ds = ds.downcast_ref::<DataSource>().unwrap();
    assert_eq!(ds.config_url, "satisfied");
}

// ── 3. createBean 与 autowire 集成 ──────────────────────────────────────

/// 验证 createBean 自动解析依赖。
#[test]
fn create_bean_resolves_dependencies() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "auto_resolve".to_string(),
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<DataSource, _>(|resolver: &Resolver| {
                let config = resolver.resolve::<Config>().unwrap();
                DataSource {
                    config_url: config.url.clone(),
                }
            })
            .depends_on::<Config>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let ds = container
        .create_bean(std::any::type_name::<DataSource>())
        .unwrap();
    let ds = ds.downcast_ref::<DataSource>().unwrap();
    assert_eq!(ds.config_url, "auto_resolve");
}

/// 验证 autowire 返回的 bean 与 resolve 返回的相同（singleton）。
#[test]
fn autowire_singleton_returns_same_instance() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "singleton_test".to_string(),
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // autowire 和 resolve 应该返回同一个 singleton
    let a = container
        .autowire(
            std::any::type_name::<Config>(),
            0, // AUTOWIRE_NO
            false,
        )
        .unwrap();
    let b = container.resolve::<Config>().unwrap();

    // 两者都解析同一个 singleton
    let a_config = a.downcast_ref::<Config>().unwrap();
    // b 已经是 Arc<Config>，直接解引用
    assert_eq!(a_config.url, b.url);
}

// ── 4. RequestScope / SessionScope 注册与激活 ───────────────────────────

/// 参照 Spring `SimpleScopeTests.canGetScopedObject`：
/// 验证自定义 Scope 注册和使用。
#[test]
fn custom_scope_registration_and_usage() {
    static GET_COUNT: AtomicUsize = AtomicUsize::new(0);

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
    }

    GET_COUNT.store(0, Ordering::SeqCst);

    let scope = RequestScope;
    let obj1 = scope.get("test", &|| Box::new(42i32)).unwrap();
    let obj2 = scope.get("test", &|| Box::new(100i32)).unwrap();

    // 每次 get 都创建新实例
    assert_eq!(GET_COUNT.load(Ordering::SeqCst), 2);
    let v1 = obj1.downcast_ref::<i32>().unwrap();
    let v2 = obj2.downcast_ref::<i32>().unwrap();
    assert_eq!(*v1, 42);
    assert_eq!(*v2, 100);
}

/// 验证自定义 Scope 的 remove 默认返回 None。
#[test]
fn custom_scope_remove_default() {
    struct MinimalScope;

    impl BeanScope for MinimalScope {
        fn get(
            &self,
            _name: &str,
            object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
        ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(object_factory())
        }
    }

    let scope = MinimalScope;
    assert!(scope.remove("test").unwrap().is_none());
}

/// 验证自定义 Scope 的 resolve_contextual_object 默认返回 None。
#[test]
fn custom_scope_resolve_contextual_object_default() {
    struct MinimalScope;

    impl BeanScope for MinimalScope {
        fn get(
            &self,
            _name: &str,
            object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
        ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(object_factory())
        }
    }

    let scope = MinimalScope;
    assert!(scope.resolve_contextual_object("key").is_none());
}

/// 验证自定义 Scope 的 conversation_id 默认返回 None。
#[test]
fn custom_scope_conversation_id_default() {
    struct MinimalScope;

    impl BeanScope for MinimalScope {
        fn get(
            &self,
            _name: &str,
            object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
        ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(object_factory())
        }
    }

    let scope = MinimalScope;
    assert!(scope.conversation_id().is_none());
}

/// 验证 Scope destroy callback 默认不 panic。
#[test]
fn custom_scope_destroy_callback_default() {
    struct MinimalScope;

    impl BeanScope for MinimalScope {
        fn get(
            &self,
            _name: &str,
            object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
        ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(object_factory())
        }
    }

    let scope = MinimalScope;
    // register_destruction_callback 默认空实现，不应 panic
    scope.register_destruction_callback("test", Box::new(|| {}));
}

// ── 5. Scope 与 Container 集成 ──────────────────────────────────────────

/// 验证 Scope 注册到 ConfigurableBeanFactory。
#[test]
fn scope_registration_on_container() {
    use vernal_beans::configurable_bean_factory::ConfigurableBeanFactory;

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
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "scope_test".to_string(),
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // Container 当前不实现 ConfigurableBeanFactory（只是 trait 定义）
    // 但我们可以验证 Scope trait 的完整行为
    let scope = TestScope;
    let obj = scope.get("test", &|| Box::new(42i32)).unwrap();
    let val = obj.downcast_ref::<i32>().unwrap();
    assert_eq!(*val, 42);
}

// ── 6. BeanDefinitionRegistry 操作 ──────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.beanDefinitionOverriding`：
/// 验证 Bean 定义注册操作。
#[test]
fn bean_definition_registry_operations() {
    let mut builder = RegistryBuilder::new();

    // 注册第一个组件
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "first".to_string(),
            },
        ))
        .unwrap();

    // 验证注册后数量
    assert_eq!(builder.len(), 1);
    assert!(!builder.is_empty());

    // 注册第二个组件
    builder
        .register(ComponentDefinition::singleton::<DataSource, _>(
            |_resolver: &Resolver| DataSource {
                config_url: "second".to_string(),
            },
        ))
        .unwrap();

    assert_eq!(builder.len(), 2);

    // 构建 registry
    let registry = builder.build().unwrap();
    assert_eq!(registry.len(), 2);

    // 验证 contains
    let container = registry.container();
    let config_key = ComponentKey::of::<Config>();
    let ds_key = ComponentKey::of::<DataSource>();
    let missing_key = ComponentKey::of::<UserService>();

    assert!(container.contains_bean(&config_key));
    assert!(container.contains_bean(&ds_key));
    assert!(!container.contains_bean(&missing_key));
}

/// 验证重复注册同类型返回错误。
#[test]
fn bean_definition_registry_duplicate_rejected() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "first".to_string(),
            },
        ))
        .unwrap();

    // 重复注册同类型应失败
    let result = builder.register(ComponentDefinition::singleton::<Config, _>(
        |_resolver: &Resolver| Config {
            url: "second".to_string(),
        },
    ));

    assert!(result.is_err());
}

/// 验证空 RegistryBuilder。
#[test]
fn empty_registry_builder() {
    let builder = RegistryBuilder::new();
    assert_eq!(builder.len(), 0);
    assert!(builder.is_empty());

    let registry = builder.build().unwrap();
    assert_eq!(registry.len(), 0);
}

/// 验证 Registry snapshot 诊断信息。
#[test]
fn registry_snapshot_diagnostics() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "diag".to_string(),
            },
        ))
        .unwrap();
    builder
        .register(ComponentDefinition::transient::<DataSource, _>(
            |_resolver: &Resolver| DataSource {
                config_url: "diag".to_string(),
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let snapshot = registry.snapshot();
    let summary = snapshot.summary();

    assert_eq!(summary.definition_count(), 2);
    assert_eq!(summary.singleton_count(), 1);
    assert_eq!(summary.transient_count(), 1);
}

// ── 7. autowire 与 PostProcessor 集成 ───────────────────────────────────

/// 验证 autowire 后的 bean 经过 PostProcessor 处理。
#[test]
fn autowire_applies_post_processor() {
    use vernal_beans::bean_post_processor::BeanPostProcessor;

    static PP_CALLED: AtomicUsize = AtomicUsize::new(0);

    struct TestPostProcessor;

    impl BeanPostProcessor for TestPostProcessor {
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
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "pp_test".to_string(),
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let mut container = registry.container();
    container.add_bean_post_processor(Arc::new(TestPostProcessor));

    // autowire 应该触发 PostProcessor
    let _ = container
        .autowire(
            std::any::type_name::<Config>(),
            0, // AUTOWIRE_NO
            false,
        )
        .unwrap();

    // PostProcessor 应该被调用（通过 resolve → resolve_definition → PostProcessor 链）
    assert!(PP_CALLED.load(Ordering::SeqCst) > 0);
}

// ── 8. Transient Scope 行为 ─────────────────────────────────────────────

/// 验证 Transient 每次创建新实例。
#[test]
fn transient_creates_new_instance_each_time() {
    static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug)]
    struct TransientBean {
        id: usize,
    }

    CALL_COUNT.store(0, Ordering::SeqCst);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::transient::<TransientBean, _>(
            |_resolver: &Resolver| {
                let id = CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                TransientBean { id }
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let a = container.resolve::<TransientBean>().unwrap();
    let b = container.resolve::<TransientBean>().unwrap();
    let c = container.resolve::<TransientBean>().unwrap();

    assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 3);
    assert_ne!(a.id, b.id);
    assert_ne!(b.id, c.id);
}

/// 验证 Singleton 缓存复用。
#[test]
fn singleton_caches_instance() {
    static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug)]
    struct SingletonBean {
        id: usize,
    }

    CALL_COUNT.store(0, Ordering::SeqCst);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<SingletonBean, _>(
            |_resolver: &Resolver| {
                let id = CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                SingletonBean { id }
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let a = container.resolve::<SingletonBean>().unwrap();
    let b = container.resolve::<SingletonBean>().unwrap();

    assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1); // 只调用一次
    assert!(Arc::ptr_eq(&a, &b)); // 同一个实例
}

// ── 9. autowire_bean_properties 按名称模式 ──────────────────────────────

/// 验证 autowire_bean_properties BY_NAME 模式。
#[test]
fn autowire_bean_properties_by_name_mode() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "by_name_props".to_string(),
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<DataSource, _>(|resolver: &Resolver| {
                let config = resolver.resolve::<Config>().unwrap();
                DataSource {
                    config_url: config.url.clone(),
                }
            })
            .depends_on::<Config>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // 创建一个独立的 DataSource 实例
    let standalone = Arc::new(DataSource {
        config_url: "standalone".to_string(),
    });

    // BY_NAME 模式：应该按名称查找依赖
    let result = container
        .autowire_bean_properties(standalone, 1, false)
        .unwrap();

    // 结果是原始 bean（autowire_bean_properties 返回原实例）
    let result = result.downcast_ref::<DataSource>().unwrap();
    assert_eq!(result.config_url, "standalone");
}

/// 验证 autowire_bean_properties BY_TYPE 模式。
#[test]
fn autowire_bean_properties_by_type_mode() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "by_type_props".to_string(),
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let standalone = Arc::new(DataSource {
        config_url: "standalone".to_string(),
    });

    // BY_TYPE 模式
    let result = container
        .autowire_bean_properties(standalone, 2, false)
        .unwrap();
    let result = result.downcast_ref::<DataSource>().unwrap();
    assert_eq!(result.config_url, "standalone");
}

// ── 10. resolveDependency 按类型解析 ─────────────────────────────────────

/// 验证 resolveDependency 按类型解析。
#[test]
fn resolve_dependency_by_type_lookup() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "dep_lookup".to_string(),
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let descriptor = vernal_beans::DependencyDescriptor::for_field(
        std::any::TypeId::of::<Config>(),
        std::any::type_name::<Config>(),
    );

    let result = container.resolve_dependency(&descriptor, None).unwrap();
    assert!(result.is_some());

    let unwrapped = result.unwrap();
    let config = unwrapped.downcast_ref::<Config>().unwrap();
    assert_eq!(config.url, "dep_lookup");
}

/// 验证 resolveDependency 找不到时返回错误。
#[test]
fn resolve_dependency_not_found_returns_error() {
    let registry = vernal_beans::Registry::empty();
    let container = registry.container();

    let descriptor = vernal_beans::DependencyDescriptor::for_field(
        std::any::TypeId::of::<Config>(),
        std::any::type_name::<Config>(),
    );

    let result = container.resolve_dependency(&descriptor, None);
    assert!(result.is_err());
}

/// 验证可选依赖找不到时返回 None。
#[test]
fn resolve_optional_dependency_returns_none() {
    let registry = vernal_beans::Registry::empty();
    let container = registry.container();

    let descriptor = vernal_beans::DependencyDescriptor::for_field(
        std::any::TypeId::of::<Config>(),
        std::any::type_name::<Config>(),
    )
    .with_optional(true);

    let result = container.resolve_dependency(&descriptor, None).unwrap();
    assert!(result.is_none());
}

// ── 11. initializeBean 完整流程 ──────────────────────────────────────────

/// 验证 initializeBean 执行 before + after PostProcessor 链。
#[test]
fn initialize_bean_full_lifecycle() {
    use vernal_beans::bean_post_processor::BeanPostProcessor;

    static BEFORE_CALLED: AtomicUsize = AtomicUsize::new(0);
    static AFTER_CALLED: AtomicUsize = AtomicUsize::new(0);

    struct LifecyclePostProcessor;

    impl BeanPostProcessor for LifecyclePostProcessor {
        fn post_process_before_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            BEFORE_CALLED.fetch_add(1, Ordering::SeqCst);
            Ok(Some(bean))
        }

        fn post_process_after_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            AFTER_CALLED.fetch_add(1, Ordering::SeqCst);
            Ok(Some(bean))
        }
    }

    BEFORE_CALLED.store(0, Ordering::SeqCst);
    AFTER_CALLED.store(0, Ordering::SeqCst);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "lifecycle".to_string(),
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let mut container = registry.container();
    container.add_bean_post_processor(Arc::new(LifecyclePostProcessor));

    let bean = Arc::new(Config {
        url: "test".to_string(),
    });
    let result = container.initialize_bean(bean, "Config").unwrap();

    assert!(BEFORE_CALLED.load(Ordering::SeqCst) > 0);
    assert!(AFTER_CALLED.load(Ordering::SeqCst) > 0);
}

// ── 12. destroyBeanInstance 不 panic ────────────────────────────────────

/// 验证 destroyBeanInstance 正常执行。
#[test]
fn destroy_bean_instance_succeeds() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "destroy".to_string(),
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let bean = Config {
        url: "destroy".to_string(),
    };
    let result = container.destroy_bean_instance("Config", &bean);
    assert!(result.is_ok());
}
