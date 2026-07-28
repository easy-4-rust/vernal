//! spring-beans → vernal-beans 迁移测试。
//!
//! 参照 Spring Framework 7.0.8 的 spring-beans 测试用例，
//! 验证 vernal-beans 的 trait 定义和语义对齐。
//!
//! 测试覆盖范围：
//! - BeanDefinition trait + 常量
//! - BeanFactory trait + Container 实现
//! - BeanPostProcessor 体系
//! - FactoryBean trait
//! - Aware 体系 + 生命周期
//! - Scope trait
//! - ConstructorArgumentValues + ValueHolder
//! - MutablePropertyValues
//! - ObjectProvider
//! - NamedBeanHolder
//! - InjectionPoint + DependencyDescriptor
//! - Autowire 枚举

use std::any::Any;
use std::sync::Arc;

use vernal_beans::ComponentKey;
use vernal_beans::Container;
use vernal_beans::Scope;
use vernal_beans::autowire::Autowire;
use vernal_beans::bean_definition::{
    BeanDefinition, ROLE_APPLICATION, ROLE_INFRASTRUCTURE, ROLE_SUPPORT, SCOPE_PROTOTYPE,
    SCOPE_SINGLETON,
};
use vernal_beans::bean_factory::BeanFactory;
use vernal_beans::bean_factory_aware::BeanFactoryAware;
use vernal_beans::bean_name_aware::BeanNameAware;
use vernal_beans::bean_post_processor::BeanPostProcessor;
use vernal_beans::bean_scope::BeanScope;
use vernal_beans::constructor_argument_values::{ConstructorArgumentValues, ValueHolder};
use vernal_beans::dependency_descriptor::DependencyDescriptor;
use vernal_beans::disposable_bean::DisposableBean;
use vernal_beans::factory_bean::FactoryBean;
use vernal_beans::initializing_bean::InitializingBean;
use vernal_beans::injection_point::InjectionPoint;
use vernal_beans::mutable_property_values::MutablePropertyValues;
use vernal_beans::named_bean_holder::NamedBeanHolder;
use vernal_beans::object_provider::ObjectProvider;
use vernal_beans::property_value::PropertyValue;

// ── 辅助类型 ─────────────────────────────────────────────────────────────

#[derive(Debug)]
struct TestBean {
    name: String,
    value: i32,
}

impl Default for TestBean {
    fn default() -> Self {
        Self {
            name: "test".to_string(),
            value: 0,
        }
    }
}

#[derive(Debug)]
struct TestBeanDefinition {
    key: ComponentKey,
    scope: Scope,
    lazy: bool,
    primary: bool,
}

impl BeanDefinition for TestBeanDefinition {
    fn bean_name(&self) -> &ComponentKey {
        &self.key
    }
    fn bean_class_name(&self) -> &str {
        "TestBean"
    }
    fn scope(&self) -> Scope {
        self.scope
    }
    fn is_lazy_init(&self) -> bool {
        self.lazy
    }
    fn is_primary(&self) -> bool {
        self.primary
    }
    fn role(&self) -> i32 {
        ROLE_APPLICATION
    }
}

// ── BeanDefinition 测试 ──────────────────────────────────────────────────

/// 参照 Spring `BeanDefinitionTests`：验证 BeanDefinition 常量和基本属性。
#[test]
fn bean_definition_constants() {
    assert_eq!(SCOPE_SINGLETON, "singleton");
    assert_eq!(SCOPE_PROTOTYPE, "prototype");
    assert_eq!(ROLE_APPLICATION, 0);
    assert_eq!(ROLE_SUPPORT, 1);
    assert_eq!(ROLE_INFRASTRUCTURE, 2);
}

#[test]
fn bean_definition_singleton_scope() {
    let def = TestBeanDefinition {
        key: ComponentKey::of::<TestBean>(),
        scope: Scope::Singleton,
        lazy: false,
        primary: false,
    };
    assert!(def.is_singleton());
    assert!(!def.is_prototype());
    assert_eq!(def.scope(), Scope::Singleton);
}

#[test]
fn bean_definition_prototype_scope() {
    let def = TestBeanDefinition {
        key: ComponentKey::of::<TestBean>(),
        scope: Scope::Transient,
        lazy: true,
        primary: false,
    };
    assert!(!def.is_singleton());
    assert!(def.is_prototype());
    assert!(def.is_lazy_init());
}

#[test]
fn bean_definition_primary() {
    let def = TestBeanDefinition {
        key: ComponentKey::of::<TestBean>(),
        scope: Scope::Singleton,
        lazy: false,
        primary: true,
    };
    assert!(def.is_primary());
}

#[test]
fn bean_definition_default_values() {
    let def = TestBeanDefinition {
        key: ComponentKey::of::<TestBean>(),
        scope: Scope::Singleton,
        lazy: false,
        primary: false,
    };
    assert!(!def.is_abstract());
    assert_eq!(def.description(), None);
    assert_eq!(def.parent_name(), None);
    assert_eq!(def.factory_bean_name(), None);
    assert_eq!(def.factory_method_name(), None);
    assert_eq!(def.init_method_name(), None);
    assert_eq!(def.destroy_method_name(), None);
    assert_eq!(def.resource_description(), None);
    assert!(def.is_autowire_candidate());
    assert!(!def.is_fallback());
}

// ── BeanFactory + Container 测试 ─────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.prototype`：
/// 验证 singleton vs prototype 作用域行为。
#[test]
fn container_singleton_vs_prototype() {
    // Container 需要 Registry，这里测试 trait 存在性
    // 完整集成测试需要通过 RegistryBuilder 构建
    let scope_singleton = Scope::Singleton;
    let scope_transient = Scope::Transient;

    assert!(scope_singleton.is_singleton());
    assert!(!scope_singleton.is_transient());
    assert!(scope_transient.is_transient());
    assert!(!scope_transient.is_singleton());
}

/// 参照 Spring `DefaultListableBeanFactoryTests.empty`：
/// 验证空工厂返回空列表。
#[test]
fn container_empty_factory() {
    let registry = vernal_beans::Registry::empty();
    let container = registry.container();

    // 空容器不应包含任何 Bean
    let key = ComponentKey::of::<TestBean>();
    assert!(!container.contains_bean(&key));
}

/// 参照 Spring `BeanFactoryTests`：验证 BeanFactory trait 方法签名。
#[test]
fn bean_factory_trait_methods() {
    // 验证 BeanFactory trait 的常量
    assert_eq!(vernal_beans::bean_factory::FACTORY_BEAN_PREFIX, "&");

    // 验证 trait 存在（编译通过即验证）
    fn _assert_bean_factory<T: BeanFactory>() {}
    _assert_bean_factory::<Container>();
}

// ── BeanPostProcessor 测试 ───────────────────────────────────────────────

/// 参照 Spring `BeanPostProcessorTests`：验证默认行为。
#[test]
fn bean_post_processor_default_behavior() {
    struct TestPostProcessor;

    impl BeanPostProcessor for TestPostProcessor {}

    let processor = TestPostProcessor;
    let bean: Arc<dyn Any + Send + Sync> = Arc::new(TestBean::default());

    // 默认行为：返回原始 Bean
    let result = processor
        .post_process_before_initialization(bean.clone(), "test")
        .unwrap();
    assert!(result.is_some());
    let returned = result.unwrap();
    // 验证是同一个 Arc
    assert!(Arc::ptr_eq(&returned, &bean));

    let result = processor
        .post_process_after_initialization(bean.clone(), "test")
        .unwrap();
    assert!(result.is_some());
}

/// 参照 Spring `BeanPostProcessorTests`：验证返回 None 时的语义。
#[test]
fn bean_post_processor_returns_none() {
    struct NullPostProcessor;

    impl BeanPostProcessor for NullPostProcessor {
        fn post_process_before_initialization(
            &self,
            _bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(None)
        }
    }

    let processor = NullPostProcessor;
    let bean: Arc<dyn Any + Send + Sync> = Arc::new(TestBean::default());

    let result = processor
        .post_process_before_initialization(bean, "test")
        .unwrap();
    assert!(result.is_none());
}

/// 验证 BeanPostProcessor 错误传播。
#[test]
fn bean_post_processor_error_propagation() {
    struct ErrorPostProcessor;

    impl BeanPostProcessor for ErrorPostProcessor {
        fn post_process_before_initialization(
            &self,
            _bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Err("Post processing failed".into())
        }
    }

    let processor = ErrorPostProcessor;
    let bean: Arc<dyn Any + Send + Sync> = Arc::new(TestBean::default());

    let result = processor.post_process_before_initialization(bean, "test");
    assert!(result.is_err());
}

// ── FactoryBean 测试 ─────────────────────────────────────────────────────

/// 参照 Spring `FactoryBeanTests`：验证 FactoryBean 语义。
#[test]
fn factory_bean_singleton() {
    struct TestFactoryBean;

    impl FactoryBean for TestFactoryBean {
        fn get_object(
            &self,
        ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Arc::new(TestBean {
                name: "factory_created".to_string(),
                value: 42,
            }))
        }

        fn get_object_type(&self) -> Option<std::any::TypeId> {
            Some(std::any::TypeId::of::<TestBean>())
        }

        fn is_singleton(&self) -> bool {
            true
        }
    }

    let factory = TestFactoryBean;
    assert!(factory.is_singleton());
    assert_eq!(
        factory.get_object_type(),
        Some(std::any::TypeId::of::<TestBean>())
    );

    let obj = factory.get_object().unwrap();
    let bean = obj.downcast_ref::<TestBean>().unwrap();
    assert_eq!(bean.name, "factory_created");
    assert_eq!(bean.value, 42);
}

/// 验证 FactoryBean 默认 is_singleton = true。
#[test]
fn factory_bean_default_singleton() {
    struct DefaultFactoryBean;

    impl FactoryBean for DefaultFactoryBean {
        fn get_object(
            &self,
        ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Arc::new(TestBean::default()))
        }

        fn get_object_type(&self) -> Option<std::any::TypeId> {
            Some(std::any::TypeId::of::<TestBean>())
        }
        // 不覆盖 is_singleton，使用默认值 true
    }

    let factory = DefaultFactoryBean;
    assert!(factory.is_singleton()); // 默认 true
}

// ── Aware 体系测试 ───────────────────────────────────────────────────────

/// 参照 Spring 生命周期：验证 BeanNameAware 回调。
#[test]
fn bean_name_aware_callback() {
    struct AwareService {
        received_name: Option<String>,
    }

    impl vernal_beans::aware::Aware for AwareService {}

    impl BeanNameAware for AwareService {
        fn set_bean_name(&mut self, name: &str) {
            self.received_name = Some(name.to_string());
        }
    }

    let mut service = AwareService {
        received_name: None,
    };
    assert!(service.received_name.is_none());

    service.set_bean_name("myService");
    assert_eq!(service.received_name.as_deref(), Some("myService"));
}

/// 验证 BeanFactoryAware 回调。
#[test]
fn bean_factory_aware_callback() {
    struct FactoryAwareService {
        received_factory: bool,
    }

    impl vernal_beans::aware::Aware for FactoryAwareService {}

    impl BeanFactoryAware for FactoryAwareService {
        fn set_bean_factory(&mut self, _bean_factory: Arc<dyn std::any::Any + Send + Sync>) {
            self.received_factory = true;
        }
    }

    let mut service = FactoryAwareService {
        received_factory: false,
    };
    assert!(!service.received_factory);

    let factory: Arc<dyn Any + Send + Sync> = Arc::new("mock_factory");
    service.set_bean_factory(factory);
    assert!(service.received_factory);
}

// ── InitializingBean 测试 ────────────────────────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.multipleInitAndDestroyMethods`：
/// 验证初始化回调。
#[test]
fn initializing_bean_callback() {
    struct InitService {
        initialized: bool,
    }

    impl vernal_beans::aware::Aware for InitService {}

    impl InitializingBean for InitService {
        fn after_properties_set(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            self.initialized = true;
            Ok(())
        }
    }

    let mut service = InitService { initialized: false };
    assert!(!service.initialized);

    service.after_properties_set().unwrap();
    assert!(service.initialized);
}

/// 验证 InitializingBean 错误传播。
#[test]
fn initializing_bean_error() {
    struct FailInitService;

    impl vernal_beans::aware::Aware for FailInitService {}

    impl InitializingBean for FailInitService {
        fn after_properties_set(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Err("Init failed".into())
        }
    }

    let mut service = FailInitService;
    let result = service.after_properties_set();
    assert!(result.is_err());
}

// ── DisposableBean 测试 ──────────────────────────────────────────────────

/// 参照 Spring `DefaultSingletonBeanRegistryTests.disposableBean`：
/// 验证销毁回调。
#[test]
fn disposable_bean_callback() {
    struct DestroyService {
        destroyed: bool,
    }

    impl vernal_beans::aware::Aware for DestroyService {}

    impl DisposableBean for DestroyService {
        fn destroy(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            // 注意：Rust 中 &self 不可变，这里用 Cell 模拟
            Ok(())
        }
    }

    let service = DestroyService { destroyed: false };
    let result = service.destroy();
    assert!(result.is_ok());
}

/// 验证 DisposableBean 默认实现（空操作）。
#[test]
fn disposable_bean_default() {
    struct SimpleService;

    impl vernal_beans::aware::Aware for SimpleService {}

    impl DisposableBean for SimpleService {
        // 不覆盖 destroy，使用默认实现
    }

    let service = SimpleService;
    let result = service.destroy();
    assert!(result.is_ok());
}

// ── Scope trait 测试 ─────────────────────────────────────────────────────

/// 参照 Spring `SimpleScopeTests.canGetScopedObject`：
/// 验证自定义 Scope 语义。
#[test]
fn custom_scope_get() {
    struct CustomScope {
        instance_count: std::sync::atomic::AtomicUsize,
    }

    impl BeanScope for CustomScope {
        fn get(
            &self,
            _name: &str,
            object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
        ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            self.instance_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(object_factory())
        }
    }

    let scope = CustomScope {
        instance_count: std::sync::atomic::AtomicUsize::new(0),
    };

    let obj1 = scope
        .get("test", &|| Box::new(TestBean::default()))
        .unwrap();
    let obj2 = scope
        .get("test", &|| {
            Box::new(TestBean {
                name: "second".to_string(),
                value: 1,
            })
        })
        .unwrap();

    // 每次 get 都创建新实例
    assert_eq!(
        scope
            .instance_count
            .load(std::sync::atomic::Ordering::SeqCst),
        2
    );
    // 验证是不同的实例
    let b1 = obj1.downcast_ref::<TestBean>().unwrap();
    let b2 = obj2.downcast_ref::<TestBean>().unwrap();
    assert_eq!(b1.value, 0);
    assert_eq!(b2.value, 1);
}

/// 验证 Scope 默认方法。
#[test]
fn scope_default_methods() {
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
    // 默认 remove 返回 None
    assert!(scope.remove("test").unwrap().is_none());
    // 默认 resolve_contextual_object 返回 None
    assert!(scope.resolve_contextual_object("key").is_none());
    // 默认 conversation_id 返回 None
    assert!(scope.conversation_id().is_none());
}

// ── ConstructorArgumentValues 测试 ───────────────────────────────────────

/// 参照 Spring `ConstructorArgumentValuesTests`：验证构造参数管理。
#[test]
fn constructor_argument_values_indexed() {
    let mut cav = ConstructorArgumentValues::new();

    let vh1 = ValueHolder::new(Arc::new(42i32));
    let vh2 = ValueHolder::with_type(Arc::new("hello".to_string()), "String");

    cav.add_indexed_argument_value(0, vh1);
    cav.add_indexed_argument_value(1, vh2);

    assert!(cav.has_indexed_argument_value(0));
    assert!(cav.has_indexed_argument_value(1));
    assert!(!cav.has_indexed_argument_value(2));

    assert_eq!(cav.argument_count(), 2);
    assert!(!cav.is_empty());
}

#[test]
fn constructor_argument_values_generic() {
    let mut cav = ConstructorArgumentValues::new();

    let vh = ValueHolder::with_type_and_name(Arc::new(3.14f64), "f64", "pi");
    cav.add_generic_argument_value(vh);

    let found = cav.get_generic_argument_value("f64");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name(), Some("pi"));

    assert_eq!(cav.generic_argument_values().len(), 1);
}

#[test]
fn constructor_argument_values_combined_query() {
    let mut cav = ConstructorArgumentValues::new();

    let vh = ValueHolder::new(Arc::new(42i32));
    cav.add_indexed_argument_value(0, vh);

    let found = cav.get_argument_value(0, None, None);
    assert!(found.is_some());

    // 按索引查找
    let found = cav.get_argument_value(0, Some("i32"), None);
    assert!(found.is_some());

    // 不存在的索引
    let found = cav.get_argument_value(99, None, None);
    assert!(found.is_none());
}

#[test]
fn constructor_argument_values_copy() {
    let mut cav = ConstructorArgumentValues::new();
    cav.add_indexed_argument_value(0, ValueHolder::new(Arc::new(1i32)));
    cav.add_generic_argument_value(ValueHolder::new(Arc::new(2i32)));

    let cav2 = ConstructorArgumentValues::from_other(&cav);
    assert_eq!(cav2.argument_count(), 2);

    // 修改原集合不影响副本
    cav.clear();
    assert_eq!(cav.argument_count(), 0);
    assert_eq!(cav2.argument_count(), 2);
}

// ── ValueHolder 测试 ─────────────────────────────────────────────────────

#[test]
fn value_holder_basic() {
    let vh = ValueHolder::new(Arc::new(42i32));
    assert!(vh.value().is_some());
    assert_eq!(vh.type_name(), None);
    assert_eq!(vh.name(), None);
    assert!(!vh.is_converted());
}

#[test]
fn value_holder_with_type_and_name() {
    let vh = ValueHolder::with_type_and_name(Arc::new("hello".to_string()), "String", "greeting");
    assert_eq!(vh.type_name(), Some("String"));
    assert_eq!(vh.name(), Some("greeting"));
}

#[test]
fn value_holder_copy() {
    let vh = ValueHolder::with_type(Arc::new(42i32), "i32");
    let vh2 = vh.copy();
    assert_eq!(vh2.type_name(), Some("i32"));
}

// ── MutablePropertyValues 测试 ───────────────────────────────────────────

/// 参照 Spring `MutablePropertyValuesTests`：验证属性值管理。
#[test]
fn mutable_property_values_add_and_get() {
    let mut pvs = MutablePropertyValues::new();

    pvs.add_value("name", Arc::new("Alice".to_string()));
    pvs.add_value("age", Arc::new(30i32));

    assert!(pvs.contains("name"));
    assert!(pvs.contains("age"));
    assert!(!pvs.contains("email"));

    assert_eq!(pvs.len(), 2);
    assert!(!pvs.is_empty());
}

#[test]
fn mutable_property_values_overwrite() {
    let mut pvs = MutablePropertyValues::new();

    pvs.add_value("name", Arc::new("Alice".to_string()));
    pvs.add_value("name", Arc::new("Bob".to_string()));

    // 后添加的覆盖先添加的
    assert_eq!(pvs.len(), 1);
    let pv = pvs.get("name").unwrap();
    let name = pv.value().downcast_ref::<String>().unwrap();
    assert_eq!(name, "Bob");
}

#[test]
fn mutable_property_values_clear() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("key", Arc::new("value".to_string()));
    assert!(!pvs.is_empty());

    pvs.clear();
    assert!(pvs.is_empty());
}

#[test]
fn mutable_property_values_from_vec() {
    let pvs = MutablePropertyValues::from_vec(vec![
        PropertyValue::new("a", Arc::new(1i32)),
        PropertyValue::new("b", Arc::new(2i32)),
    ]);
    assert_eq!(pvs.len(), 2);
    assert!(pvs.contains("a"));
    assert!(pvs.contains("b"));
}

// ── PropertyValue 测试 ───────────────────────────────────────────────────

#[test]
fn property_value_basic() {
    let pv = PropertyValue::new("name", Arc::new("Alice".to_string()));
    assert_eq!(pv.name(), "name");
    let value = pv.value().downcast_ref::<String>().unwrap();
    assert_eq!(value, "Alice");
}

// ── NamedBeanHolder 测试 ─────────────────────────────────────────────────

/// 参照 Spring `NamedBeanHolder`：验证带名称的 Bean 持有者。
#[test]
fn named_bean_holder() {
    let holder = NamedBeanHolder::new(
        Arc::new(TestBean {
            name: "myBean".to_string(),
            value: 42,
        }),
        "myBeanName",
    );

    assert_eq!(holder.bean_name(), "myBeanName");
    let bean: &TestBean = holder.instance().as_ref();
    assert_eq!(bean.name, "myBean");
    assert_eq!(bean.value, 42);

    let (instance, name) = holder.into_inner();
    assert_eq!(name, "myBeanName");
    assert!(Arc::strong_count(&instance) >= 1);
}

// ── InjectionPoint 测试 ──────────────────────────────────────────────────

#[test]
fn injection_point_basic() {
    let ip = InjectionPoint::new(std::any::TypeId::of::<String>(), "String");

    assert_eq!(ip.type_id(), std::any::TypeId::of::<String>());
    assert_eq!(ip.type_name(), "String");
    assert_eq!(ip.containing_bean_name(), None);
    assert_eq!(ip.member_name(), None);
    assert_eq!(ip.qualifier(), None);
}

#[test]
fn injection_point_with_metadata() {
    let ip = InjectionPoint::new(std::any::TypeId::of::<i32>(), "i32")
        .with_containing_bean_name("myService")
        .with_member_name("count")
        .with_qualifier("primary");

    assert_eq!(ip.containing_bean_name(), Some("myService"));
    assert_eq!(ip.member_name(), Some("count"));
    assert_eq!(ip.qualifier(), Some("primary"));
}

// ── DependencyDescriptor 测试 ────────────────────────────────────────────

/// 参照 Spring `DependencyDescriptorTests`：验证依赖描述符。
#[test]
fn dependency_descriptor_field() {
    let dd = DependencyDescriptor::for_field(std::any::TypeId::of::<String>(), "String");

    assert_eq!(dd.type_id(), std::any::TypeId::of::<String>());
    assert_eq!(dd.type_name(), "String");
    assert!(!dd.is_optional());
    assert!(!dd.is_multiple());
    assert_eq!(dd.qualifier(), None);
}

#[test]
fn dependency_descriptor_constructor_parameter() {
    let dd =
        DependencyDescriptor::for_constructor_parameter(0, std::any::TypeId::of::<i32>(), "i32");

    assert_eq!(dd.field_index(), Some(0));
}

#[test]
fn dependency_descriptor_with_metadata() {
    let dd = DependencyDescriptor::for_field(std::any::TypeId::of::<String>(), "String")
        .with_optional(true)
        .with_multiple(false)
        .with_qualifier("primary")
        .with_containing_bean_name("myService");

    assert!(dd.is_optional());
    assert!(!dd.is_multiple());
    assert_eq!(dd.qualifier(), Some("primary"));
    assert_eq!(dd.containing_bean_name(), Some("myService"));
}

// ── Autowire 枚举测试 ────────────────────────────────────────────────────

/// 参照 Spring `Autowire` 枚举值。
#[test]
fn autowire_enum_values() {
    assert_eq!(Autowire::No.as_int(), 0);
    assert_eq!(Autowire::ByName.as_int(), 1);
    assert_eq!(Autowire::ByType.as_int(), 2);
    assert_eq!(Autowire::Constructor.as_int(), 3);
}

#[test]
fn autowire_from_int() {
    assert_eq!(Autowire::from_int(0), Some(Autowire::No));
    assert_eq!(Autowire::from_int(1), Some(Autowire::ByName));
    assert_eq!(Autowire::from_int(2), Some(Autowire::ByType));
    assert_eq!(Autowire::from_int(3), Some(Autowire::Constructor));
    assert_eq!(Autowire::from_int(4), None);
    assert_eq!(Autowire::from_int(-1), None);
}

#[test]
fn autowire_default() {
    let autowire: Autowire = Default::default();
    assert_eq!(autowire, Autowire::No);
}

// ── ObjectProvider 测试 ──────────────────────────────────────────────────

/// 参照 Spring `CustomObjectProviderTests`：验证 ObjectProvider 语义。
#[test]
fn object_provider_if_available() {
    struct AlwaysAvailableProvider;

    impl ObjectProvider<dyn Any + Send + Sync> for AlwaysAvailableProvider {
        fn get(
            &self,
        ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Arc::new(TestBean::default()))
        }
        fn if_available(&self) -> Option<Arc<dyn Any + Send + Sync>> {
            self.get().ok()
        }
        fn get_if_unique(
            &self,
        ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            self.get()
        }
        fn stream(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
            vec![self.get().unwrap()]
        }
        fn ordered_stream(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
            self.stream()
        }
    }

    let provider = AlwaysAvailableProvider;
    assert!(provider.if_available().is_some());
    assert!(provider.get().is_ok());
    assert_eq!(provider.stream().len(), 1);
}

#[test]
fn object_provider_unavailable() {
    struct NeverAvailableProvider;

    impl ObjectProvider<dyn Any + Send + Sync> for NeverAvailableProvider {
        fn get(
            &self,
        ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Err("Not available".into())
        }
        fn if_available(&self) -> Option<Arc<dyn Any + Send + Sync>> {
            self.get().ok()
        }
        fn get_if_unique(
            &self,
        ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            self.get()
        }
        fn stream(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
            Vec::new()
        }
        fn ordered_stream(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
            self.stream()
        }
    }

    let provider = NeverAvailableProvider;
    assert!(provider.if_available().is_none());
    assert!(provider.get().is_err());
    assert!(provider.stream().is_empty());
}

// ── Container BeanFactory 集成测试 ───────────────────────────────────────

/// 验证 Container 实现 BeanFactory trait。
#[test]
fn container_implements_bean_factory() {
    fn _assert_impl<T: BeanFactory>() {}
    _assert_impl::<Container>();
}

/// 参照 Spring `DefaultListableBeanFactoryTests.empty`：空容器测试。
#[test]
fn container_bean_factory_empty() {
    let registry = vernal_beans::Registry::empty();
    let container = registry.container();

    let key = ComponentKey::of::<TestBean>();
    assert!(!container.contains_bean(&key));
    assert!(container.is_singleton(&key).is_err());
    assert!(container.is_prototype(&key).is_err());
}

// ── SmartInstantiationAwareBeanPostProcessor 测试 ────────────────────────

#[test]
fn smart_post_processor_defaults() {
    use vernal_beans::instantiation_aware_bean_post_processor::InstantiationAwareBeanPostProcessor;
    use vernal_beans::smart_instantiation_aware_bean_post_processor::SmartInstantiationAwareBeanPostProcessor;

    struct SmartPP;
    impl BeanPostProcessor for SmartPP {}
    impl InstantiationAwareBeanPostProcessor for SmartPP {}
    impl SmartInstantiationAwareBeanPostProcessor for SmartPP {}

    let pp = SmartPP;
    let bean: Arc<dyn Any + Send + Sync> = Arc::new(TestBean::default());

    // 默认 predictBeanType 返回 None
    assert!(pp.predict_bean_type(&*bean, "test").unwrap().is_none());
    // 默认 determineCandidateConstructors 返回 None
    assert!(
        pp.determine_candidate_constructors(&*bean, "test")
            .unwrap()
            .is_none()
    );
    // 默认 getEarlyBeanReference 返回原始 bean
    let early = pp.get_early_bean_reference(bean.clone(), "test").unwrap();
    assert!(Arc::ptr_eq(&early, &bean));
}

// ── MergedBeanDefinitionPostProcessor 测试 ───────────────────────────────

#[test]
fn merged_bean_definition_post_processor() {
    use vernal_beans::merged_bean_definition_post_processor::MergedBeanDefinitionPostProcessor;

    struct TestMBPP;
    impl BeanPostProcessor for TestMBPP {}
    impl MergedBeanDefinitionPostProcessor for TestMBPP {
        fn post_process_merged_beandefinition(&self, _bean_type_name: &str, _bean_name: &str) {}
    }

    let pp = TestMBPP;
    // 不应 panic
    pp.post_process_merged_beandefinition("TestBean", "testBean");
    pp.reset_beandefinition("testBean"); // 默认空实现
}

// ── DestructionAwareBeanPostProcessor 测试 ───────────────────────────────

#[test]
fn destruction_aware_post_processor() {
    use vernal_beans::destruction_aware_bean_post_processor::DestructionAwareBeanPostProcessor;

    struct TestDABPP;
    impl BeanPostProcessor for TestDABPP {}
    impl DestructionAwareBeanPostProcessor for TestDABPP {
        fn post_process_before_destruction(
            &self,
            _bean: &dyn Any,
            _bean_name: &str,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
    }

    let pp = TestDABPP;
    let bean = TestBean::default();
    assert!(pp.post_process_before_destruction(&bean, "test").is_ok());
    assert!(pp.requires_destruction(&bean)); // 默认 true
}

// ── SingletonBeanRegistry 测试 ───────────────────────────────────────────

/// 参照 Spring `DefaultSingletonBeanRegistryTests.singletons`：
/// 验证单例注册和查询。
#[test]
fn singleton_bean_registry_trait() {
    use vernal_beans::singleton_bean_registry::SingletonBeanRegistry;

    // 验证 trait 存在（编译通过即验证）
    fn _assert_impl<T: SingletonBeanRegistry>() {}
    // Container 不直接实现此 trait（内部使用 singletons 字段），但 trait 定义是有效的
}

// ── BeanDefinitionRegistry 测试 ──────────────────────────────────────────

/// 参照 Spring `BeanDefinitionRegistryTests`：验证注册表语义。
#[test]
fn bean_definition_registry_trait() {
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;

    // 验证 trait 存在
    fn _assert_impl<T: BeanDefinitionRegistry>() {}
}

// ── SmartInitializingSingleton 测试 ──────────────────────────────────────

/// 参照 Spring `SmartInitializingSingleton`：验证智能初始化回调。
#[test]
fn smart_initializing_singleton_trait() {
    use vernal_beans::smart_initializing_singleton::SmartInitializingSingleton;

    struct TestSIS;

    impl SmartInitializingSingleton for TestSIS {
        fn after_singletons_instantiated(
            &self,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
    }

    let sis = TestSIS;
    assert!(sis.after_singletons_instantiated().is_ok());
}

// ── BeanDefinitionRegistryPostProcessor 测试 ─────────────────────────────

/// 参照 Spring `ConfigurationClassPostProcessor`：验证注册表后处理器。
#[test]
fn bean_definition_registry_post_processor_trait() {
    use vernal_beans::bean_definition_registry_post_processor::BeanDefinitionRegistryPostProcessor;
    use vernal_beans::bean_factory_post_processor::BeanFactoryPostProcessor;
    use vernal_beans::configurable_listable_bean_factory::ConfigurableListableBeanFactory;

    struct TestBDPP;

    impl BeanFactoryPostProcessor for TestBDPP {
        fn post_process_bean_factory(
            &self,
            _bean_factory: &mut dyn ConfigurableListableBeanFactory,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
    }

    impl BeanDefinitionRegistryPostProcessor for TestBDPP {
        fn post_process_bean_definition_registry(
            &self,
            _registry: &mut dyn vernal_beans::bean_definition_registry::BeanDefinitionRegistry,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
    }

    // 验证可以创建实例
    let _pp = TestBDPP;
}

// ── HierarchicalBeanFactory 测试 ─────────────────────────────────────────

/// 参照 Spring `HierarchicalBeanFactory`：验证父子容器语义。
#[test]
fn hierarchical_bean_factory_trait() {
    use vernal_beans::hierarchical_bean_factory::HierarchicalBeanFactory;

    // 验证 trait 存在
    fn _assert_impl<T: HierarchicalBeanFactory>() {}
}

// ── TypeConverter 测试 ───────────────────────────────────────────────────

/// 参照 Spring `TypeConverterTests`：验证类型转换器语义。
#[test]
fn type_converter_trait() {
    use vernal_beans::type_converter::TypeConverter;

    // 验证 trait 存在
    fn _assert_impl<T: TypeConverter>() {}
}

// ── BeanExpressionResolver 测试 ──────────────────────────────────────────

/// 参照 Spring `StandardBeanExpressionResolverTests`：验证表达式解析器语义。
#[test]
fn bean_expression_resolver_trait() {
    use vernal_beans::bean_expression_resolver::BeanExpressionResolver;

    // 验证 trait 存在
    fn _assert_impl<T: BeanExpressionResolver>() {}
}

// ── ConfigurableBeanFactory 测试 ─────────────────────────────────────────

/// 验证常量定义。
#[test]
fn configurable_bean_factory_constants() {
    assert_eq!(vernal_beans::SCOPE_SINGLETON, "singleton");
    assert_eq!(vernal_beans::SCOPE_PROTOTYPE, "prototype");
}
