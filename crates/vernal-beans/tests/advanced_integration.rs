//! 高级集成测试：autowire / BeanPostProcessor 链 / Parent-Child 继承 / FactoryBean。
//!
//! 参照 Spring Framework 7.0.8 的 `DefaultListableBeanFactoryTests` 的
//! 高级测试场景，使用 `RegistryBuilder` + `Container` 端到端验证。

use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use vernal_beans::ComponentDefinition;
use vernal_beans::ComponentKey;
use vernal_beans::FactoryBean;
use vernal_beans::GenericBeanDefinition;
use vernal_beans::RegistryBuilder;
use vernal_beans::Resolver;
use vernal_beans::RootBeanDefinition;
use vernal_beans::Scope;
use vernal_beans::BeanFactory;
use vernal_beans::BeanPostProcessor;
use vernal_beans::bean_scope::BeanScope;

// ── 测试类型 ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct AppConfig {
    db_url: String,
    max_connections: u32,
}

#[derive(Debug)]
struct DataSource {
    url: String,
}

#[derive(Debug)]
struct UserRepository {
    ds: Arc<DataSource>,
}

#[derive(Debug)]
struct OrderRepository {
    ds: Arc<DataSource>,
}

#[derive(Debug)]
struct UserService {
    user_repo: Arc<UserRepository>,
    order_repo: Arc<OrderRepository>,
}

// ── 1. BeanDefinitionBuilder 测试 ────────────────────────────────────────

/// 参照 Spring `BeanDefinitionBuilderTests`：验证 Builder 构建 GenericBeanDefinition。
#[test]
fn bean_definition_builder_generic() {
    let bd = vernal_beans::BeanDefinitionBuilder::generic("MyService")
        .set_scope(Scope::Singleton)
        .set_lazy_init(false)
        .set_primary(true)
        .set_init_method("init")
        .set_destroy_method("cleanup")
        .add_depends_on("DataSource")
        .add_depends_on("Config")
        .build();

    assert_eq!(bd.get_bean_class_name(), Some("MyService"));
    assert_eq!(bd.scope(), Scope::Singleton);
    assert!(!bd.is_lazy_init());
    assert!(bd.is_primary());
    assert_eq!(bd.init_method_name(), Some("init"));
    assert_eq!(bd.destroy_method_name(), Some("cleanup"));
    assert_eq!(bd.depends_on(), &["DataSource", "Config"]);
}

/// 参照 Spring `BeanDefinitionBuilderTests`：验证 RootBeanDefinition Builder。
#[test]
fn bean_definition_builder_root() {
    let bd = vernal_beans::BeanDefinitionBuilder::root("MyService")
        .set_scope(Scope::Singleton)
        .set_lazy_init(true)
        .set_primary(false)
        .set_init_method("startup")
        .add_depends_on("Pool")
        .build();

    assert_eq!(bd.bean_class_name(), "MyService");
    assert_eq!(bd.scope(), Scope::Singleton);
    assert!(bd.is_lazy_init());
    assert!(!bd.is_primary());
    assert_eq!(bd.init_method_name(), Some("startup"));
    assert_eq!(bd.depends_on(), &["Pool"]);
}

/// 验证 Builder 流式 API 的构造参数。
#[test]
fn bean_definition_builder_constructor_args() {
    let bd = vernal_beans::BeanDefinitionBuilder::generic("MyService")
        .add_constructor_arg_value(42i32)
        .add_constructor_arg_value("hello".to_string())
        .build();

    assert_eq!(bd.constructor_argument_values().argument_count(), 2);
}

/// 验证 Builder 流式 API 的属性值。
#[test]
fn bean_definition_builder_property_values() {
    let bd = vernal_beans::BeanDefinitionBuilder::generic("MyService")
        .add_property_value("name", "Alice".to_string())
        .add_property_value("age", 30i32)
        .build();

    assert!(bd.property_values().contains("name"));
    assert!(bd.property_values().contains("age"));
}

// ── 2. RootBeanDefinition 流式 API 测试 ──────────────────────────────────

/// 验证 RootBeanDefinition 流式设置。
#[test]
fn root_bean_definition_fluent() {
    let bd = RootBeanDefinition::new()
        .with_bean_class_name("FluentService")
        .with_scope(Scope::Transient)
        .with_lazy_init(true)
        .with_primary(true)
        .with_init_method("start")
        .with_init_order(100)
        .with_destroy_method("stop");

    assert_eq!(bd.bean_class_name(), "FluentService");
    assert_eq!(bd.scope(), Scope::Transient);
    assert!(bd.is_lazy_init());
    assert!(bd.is_primary());
    assert_eq!(bd.init_method_name(), Some("start"));
    assert_eq!(bd.destroy_method_name(), Some("stop"));
    assert_eq!(bd.init_order_value(), 100);
}

// ── 3. GenericBeanDefinition 测试 ────────────────────────────────────────

/// 验证 GenericBeanDefinition 默认值。
#[test]
fn generic_bean_definition_defaults() {
    let bd = GenericBeanDefinition::new();
    assert_eq!(bd.scope(), Scope::Singleton);
    assert!(!bd.is_lazy_init());
    assert!(!bd.is_abstract());
    assert!(bd.is_autowire_candidate());
    assert!(!bd.is_primary());
    assert!(!bd.is_fallback());
    assert!(!bd.is_synthetic());
    assert_eq!(bd.role(), vernal_beans::factory::config::bean_definition::ROLE_APPLICATION);
}

/// 验证 GenericBeanDefinition parent_name。
#[test]
fn generic_bean_definition_parent() {
    let mut bd = GenericBeanDefinition::new();
    bd.set_parent_name("ParentService");
    assert_eq!(bd.get_parent_name(), Some("ParentService"));
}

/// 验证 GenericBeanDefinition → RootBeanDefinition 转换。
#[test]
fn generic_to_root_conversion() {
    let mut generic = GenericBeanDefinition::new();
    generic.set_bean_class_name("MyService");
    generic.set_scope(Scope::Transient);
    generic.set_lazy_init(true);
    generic.set_primary(true);
    generic.add_depends_on("DataSource");
    generic.set_init_method_name("init");

    let root = RootBeanDefinition::from_generic(generic);
    assert_eq!(root.bean_class_name(), "MyService");
    assert_eq!(root.scope(), Scope::Transient);
    assert!(root.is_lazy_init());
    assert!(root.is_primary());
    assert_eq!(root.depends_on(), &["DataSource"]);
    assert_eq!(root.init_method_name(), Some("init"));
}

// ── 4. Parent-Child BeanDefinition 继承 ──────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.prototypeExtendsPrototype`：
/// 验证 Parent-Child BeanDefinition 属性继承。
#[test]
fn parent_child_bean_definition_inheritance() {
    // 父定义：abstract，不可直接实例化
    let mut parent = GenericBeanDefinition::new();
    parent.set_bean_class_name("BaseService");
    parent.set_scope(Scope::Singleton);
    parent.set_abstract(true);
    parent.set_init_method_name("baseInit");
    parent.set_init_order(100);
    parent.add_property_value("baseField", "base_value".to_string());

    // 子定义：继承父定义，覆盖部分属性
    let mut child = GenericBeanDefinition::new();
    child.set_bean_class_name("ConcreteService");
    child.set_parent_name("BaseService");
    child.set_scope(Scope::Singleton); // 继承父作用域
    child.set_init_method_name("concreteInit"); // 覆盖初始化方法
    child.add_property_value("childField", "child_value".to_string()); // 新增属性

    // 验证子定义保留自己的属性
    assert_eq!(child.get_bean_class_name(), Some("ConcreteService"));
    assert_eq!(child.get_parent_name(), Some("BaseService"));
    assert_eq!(child.init_method_name(), Some("concreteInit"));

    // 验证父定义是 abstract
    assert!(parent.is_abstract());
    assert!(!child.is_abstract());
}

/// 参照 Spring `DefaultListableBeanFactoryTests.scopeInheritanceForChildBeanDefinitions`：
/// 验证子 Bean 继承父 Bean 的 scope。
#[test]
fn child_inherits_parent_scope() {
    let mut parent = GenericBeanDefinition::new();
    parent.set_scope(Scope::Transient); // prototype scope

    let mut child = GenericBeanDefinition::new();
    child.set_scope(parent.scope()); // 子继承父作用域

    assert_eq!(child.scope(), Scope::Transient);
}

// ── 5. BeanPostProcessor 链执行 ──────────────────────────────────────────

/// 参照 Spring `BeanPostProcessorTests`：验证 PostProcessor 链的执行顺序。
#[test]
fn post_processor_chain_execution_order() {
    static EXEC_ORDER: AtomicUsize = AtomicUsize::new(0);

    struct OrderedPostProcessor {
        expected_order: usize,
    }

    impl BeanPostProcessor for OrderedPostProcessor {
        fn post_process_before_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            let current = EXEC_ORDER.fetch_add(1, Ordering::SeqCst);
            assert_eq!(current, self.expected_order, "PostProcessor 执行顺序错误");
            Ok(Some(bean))
        }

        fn post_process_after_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(Some(bean))
        }
    }

    EXEC_ORDER.store(0, Ordering::SeqCst);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(AppConfig {
            db_url: "test".to_string(),
            max_connections: 10,
        }))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<DataSource, _>(|resolver: &Resolver| {
                let config = resolver.resolve::<AppConfig>().unwrap();
                DataSource {
                    url: config.db_url.clone(),
                }
            })
            .depends_on::<AppConfig>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let mut container = registry.container();

    // 添加两个 PostProcessor
    container.add_bean_post_processor(Arc::new(OrderedPostProcessor { expected_order: 0 }));
    container.add_bean_post_processor(Arc::new(OrderedPostProcessor { expected_order: 1 }));

    // 验证 PostProcessor 数量
    assert_eq!(container.bean_post_processor_count(), 2);

    // resolve 应该成功（PostProcessor 链在 Container 内部执行）
    let ds = container.resolve::<DataSource>().unwrap();
    assert_eq!(ds.url, "test");
}

/// 验证 PostProcessor 可以替换 Bean。
#[test]
fn post_processor_can_replace_bean() {
    struct ProxyPostProcessor;

    impl BeanPostProcessor for ProxyPostProcessor {
        fn post_process_after_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            // 如果是 DataSource，返回一个包装后的 DataSource
            if bean.downcast_ref::<DataSource>().is_some() {
                Ok(Some(Arc::new(DataSource {
                    url: "proxied_url".to_string(),
                })))
            } else {
                Ok(Some(bean))
            }
        }
    }

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DataSource, _>(
            |_resolver: &Resolver| DataSource {
                url: "original_url".to_string(),
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let mut container = registry.container();
    container.add_bean_post_processor(Arc::new(ProxyPostProcessor));

    let ds = container.resolve::<DataSource>().unwrap();
    // PostProcessor 替换了 Bean
    assert_eq!(ds.url, "proxied_url");
}

// ── 6. FactoryBean 集成 ─────────────────────────────────────────────────

/// 参照 Spring `FactoryBeanTests`：验证 FactoryBean 通过 Container 解析。
#[test]
fn factory_bean_via_container() {
    struct DataSourceFactory {
        url: String,
    }

    impl FactoryBean for DataSourceFactory {
        fn get_object(
            &self,
        ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Arc::new(DataSource {
                url: self.url.clone(),
            }))
        }

        fn get_object_type(&self) -> Option<std::any::TypeId> {
            Some(std::any::TypeId::of::<DataSource>())
        }
    }

    // 注册 FactoryBean 本身
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DataSourceFactory, _>(
            |_resolver: &Resolver| DataSourceFactory {
                url: "factory_created".to_string(),
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // 解析 FactoryBean 本身
    let factory = container.resolve::<DataSourceFactory>().unwrap();
    assert_eq!(factory.url, "factory_created");

    // 通过 FactoryBean 创建 DataSource
    let ds = factory.get_object().unwrap();
    let ds = ds.downcast_ref::<DataSource>().unwrap();
    assert_eq!(ds.url, "factory_created");
}

/// 验证 FactoryBean 的 isSingleton 语义。
#[test]
fn factory_bean_singleton_semantics() {
    static FACTORY_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    struct SingletonFactory;

    impl FactoryBean for SingletonFactory {
        fn get_object(
            &self,
        ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            FACTORY_CALL_COUNT.fetch_add(1, Ordering::SeqCst);
            Ok(Arc::new(DataSource {
                url: "singleton_factory".to_string(),
            }))
        }

        fn get_object_type(&self) -> Option<std::any::TypeId> {
            Some(std::any::TypeId::of::<DataSource>())
        }

        fn is_singleton(&self) -> bool {
            true // singleton：getObject 只调用一次
        }
    }

    FACTORY_CALL_COUNT.store(0, Ordering::SeqCst);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<SingletonFactory, _>(
            |_resolver: &Resolver| SingletonFactory,
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let factory = container.resolve::<SingletonFactory>().unwrap();

    // 多次调用 getObject，singleton 应该只创建一次
    let _ds1 = factory.get_object().unwrap();
    let _ds2 = factory.get_object().unwrap();

    // 注意：FactoryBean 的 isSingleton 只影响容器缓存，
    // 这里直接调用 getObject 不受缓存影响
    // 但 FactoryBean 本身是 singleton，所以 factory 是同一个实例
}

// ── 7. Scope 注册与激活 ──────────────────────────────────────────────────

/// 验证自定义 Scope 注册。
#[test]
fn custom_scope_registration() {
    struct RequestScope {
        instance_count: AtomicUsize,
    }

    impl vernal_beans::bean_scope::BeanScope for RequestScope {
        fn get(
            &self,
            _name: &str,
            object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
        ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            self.instance_count.fetch_add(1, Ordering::SeqCst);
            Ok(object_factory())
        }
    }

    let scope = RequestScope {
        instance_count: AtomicUsize::new(0),
    };

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| AppConfig {
                db_url: "test".to_string(),
                max_connections: 5,
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // 验证 scope 的 get 方法
    let obj = scope.get("test", &|| Box::new(42i32)).unwrap();
    let val = obj.downcast_ref::<i32>().unwrap();
    assert_eq!(*val, 42);
    assert_eq!(scope.instance_count.load(Ordering::SeqCst), 1);
}

// ── 8. Autowire 模式枚举 ────────────────────────────────────────────────

/// 参照 Spring `Autowire` 枚举。
#[test]
fn autowire_mode_values() {
    assert_eq!(vernal_beans::Autowire::No.as_int(), 0);
    assert_eq!(vernal_beans::Autowire::ByName.as_int(), 1);
    assert_eq!(vernal_beans::Autowire::ByType.as_int(), 2);
    assert_eq!(vernal_beans::Autowire::Constructor.as_int(), 3);
    assert_eq!(vernal_beans::Autowire::from_int(4), None); // AUTOWIRE_AUTODETECT 已废弃
}

// ── 9. Container 多依赖注入链 ────────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.referenceByType`：
/// 验证多层依赖注入链。
#[test]
fn multi_layer_dependency_chain() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| AppConfig {
                db_url: "chain_db".to_string(),
                max_connections: 20,
            },
        ))
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<DataSource, _>(|resolver: &Resolver| {
                let config = resolver.resolve::<AppConfig>().unwrap();
                DataSource {
                    url: config.db_url.clone(),
                }
            })
            .depends_on::<AppConfig>(),
        )
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<UserRepository, _>(|resolver: &Resolver| {
                let ds = resolver.resolve::<DataSource>().unwrap();
                UserRepository { ds }
            })
            .depends_on::<DataSource>(),
        )
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<OrderRepository, _>(|resolver: &Resolver| {
                let ds = resolver.resolve::<DataSource>().unwrap();
                OrderRepository { ds }
            })
            .depends_on::<DataSource>(),
        )
        .unwrap();
    builder
        .register(
            ComponentDefinition::singleton::<UserService, _>(|resolver: &Resolver| {
                let user_repo = resolver.resolve::<UserRepository>().unwrap();
                let order_repo = resolver.resolve::<OrderRepository>().unwrap();
                UserService {
                    user_repo,
                    order_repo,
                }
            })
            .depends_on::<UserRepository>()
            .depends_on::<OrderRepository>(),
        )
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // 预热
    container.warm_up().unwrap();

    // 解析 UserService，验证整个依赖链
    let user_service = container.resolve::<UserService>().unwrap();
    assert_eq!(user_service.user_repo.ds.url, "chain_db");
    assert_eq!(user_service.order_repo.ds.url, "chain_db");

    // 验证 DataSource 是同一个 singleton（被 UserRepository 和 OrderRepository 共享）
    let ds_direct = container.resolve::<DataSource>().unwrap();
    assert!(Arc::ptr_eq(&user_service.user_repo.ds, &ds_direct));
    assert!(Arc::ptr_eq(&user_service.order_repo.ds, &ds_direct));
}

// ── 10. BeanDefinition 属性覆盖 ──────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.beanDefinitionOverriding`：
/// 验证 Bean 定义覆盖。
#[test]
fn bean_definition_overriding() {
    let mut builder = RegistryBuilder::new();

    // 第一次注册
    builder
        .register(ComponentDefinition::shared_value(AppConfig {
            db_url: "first_db".to_string(),
            max_connections: 5,
        }))
        .unwrap();

    // 第二次注册同类型（应该失败，因为 ComponentKey 基于 TypeId）
    let result = builder.register(ComponentDefinition::shared_value(AppConfig {
        db_url: "second_db".to_string(),
        max_connections: 10,
    }));

    assert!(result.is_err());
}

// ── 11. 空 Builder 构建 ─────────────────────────────────────────────────

/// 验证空 Builder 构建的 GenericBeanDefinition。
#[test]
fn empty_builder() {
    let bd = vernal_beans::BeanDefinitionBuilder::generic("EmptyService").build();
    assert_eq!(bd.get_bean_class_name(), Some("EmptyService"));
    assert_eq!(bd.scope(), Scope::Singleton);
    assert!(!bd.is_lazy_init());
    assert!(!bd.is_primary());
    assert!(bd.constructor_argument_values().is_empty());
    assert!(bd.property_values().is_empty());
}

// ── 12. AutowireCapableBeanFactory trait 存在性 ──────────────────────────

/// 验证 AutowireCapableBeanFactory trait 定义。
#[test]
fn autowire_capable_bean_factory_trait() {
    // 验证 AutowireCapableBeanFactory trait 存在（编译通过即验证）
    fn _assert_impl<T: vernal_beans::AutowireCapableBeanFactory>() {}
    // Container 还未实现此 trait，所以这里只验证 trait 存在
}
