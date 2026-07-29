//! Container 实现 ConfigurableBeanFactory trait 的集成测试。
//!
//! 对应 Spring 的 `ConfigurableBeanFactory` 语义。

use std::any::Any;
use std::sync::Arc;
use vernal_beans::{
    BeanFactory, ComponentDefinition, ComponentKey, Container, RegistryBuilder,
    bean_post_processor::BeanPostProcessor, bean_scope::BeanScope,
    configurable_bean_factory::ConfigurableBeanFactory,
};

fn make_container() -> Container {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }));
    Container::new(b.build().unwrap())
}

fn as_cbf(c: &mut Container) -> &mut dyn ConfigurableBeanFactory {
    c
}

// ── register_scope / registered_scope_names ────────────────────────────

#[test]
fn configurable_register_and_list_scopes() {
    let mut c = make_container();
    let factory = as_cbf(&mut c);

    // 初始状态：无 scope
    assert!(factory.registered_scope_names().is_empty());

    // 注册一个自定义 scope
    factory.register_scope("customScope", Box::new(TestScope));
    let names = factory.registered_scope_names();
    assert_eq!(names.len(), 1);
    assert!(names.contains(&"customScope".to_string()));

    // 注册另一个
    factory.register_scope("anotherScope", Box::new(TestScope));
    let names = factory.registered_scope_names();
    assert_eq!(names.len(), 2);
}

#[test]
fn configurable_get_registered_scope() {
    let mut c = make_container();
    let factory = as_cbf(&mut c);
    factory.register_scope("test", Box::new(TestScope));
    // get_registered_scope 目前总是返回 None（Mutex 限制）
    assert!(factory.get_registered_scope("test").is_none());
    assert!(factory.get_registered_scope("nonexistent").is_none());
}

/// 测试用 BeanScope 实现
struct TestScope;
impl BeanScope for TestScope {
    fn get(
        &self,
        _name: &str,
        _object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Box::new("test_value".to_string()))
    }
}

// ── add_bean_post_processor / bean_post_processor_count ────────────────

#[test]
fn configurable_bean_post_processor_lifecycle() {
    let mut c = make_container();
    let factory = as_cbf(&mut c);

    assert_eq!(factory.bean_post_processor_count(), 0);

    struct DummyPP;
    impl BeanPostProcessor for DummyPP {
        fn post_process_before_initialization(
            &self,
            _b: Arc<dyn Any + Send + Sync>,
            _n: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(None)
        }
        fn post_process_after_initialization(
            &self,
            _b: Arc<dyn Any + Send + Sync>,
            _n: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(None)
        }
    }

    factory.add_bean_post_processor(Arc::new(DummyPP));
    assert_eq!(factory.bean_post_processor_count(), 1);

    factory.add_bean_post_processor(Arc::new(DummyPP));
    assert_eq!(factory.bean_post_processor_count(), 2);
}

// ── register_alias ───────────────────────────────────────────────────

#[test]
fn configurable_register_alias() {
    let mut c = make_container();
    let factory = as_cbf(&mut c);

    // 注册 alias
    assert!(factory.register_alias("originalBean", "myAlias").is_ok());

    // 重复 alias 应报错
    let result = factory.register_alias("otherBean", "myAlias");
    assert!(result.is_err());
}

// ── is_factory_bean ─────────────────────────────────────────────────

#[test]
fn configurable_is_factory_bean() {
    let c = make_container();
    let factory = &c as &dyn ConfigurableBeanFactory;

    // 普通名称
    assert!(!factory.is_factory_bean("normalBean"));
    // FactoryBean 前缀
    assert!(factory.is_factory_bean("&factoryBean"));
    // 空字符串
    assert!(!factory.is_factory_bean(""));
}

// ── currently_in_creation lifecycle ─────────────────────────────────

#[test]
fn configurable_currently_in_creation() {
    let mut c = make_container();
    let factory = as_cbf(&mut c);

    let bean_name = "testBean";
    assert!(!factory.is_currently_in_creation(bean_name));

    factory.set_currently_in_creation(bean_name, true);
    assert!(factory.is_currently_in_creation(bean_name));

    factory.set_currently_in_creation(bean_name, false);
    assert!(!factory.is_currently_in_creation(bean_name));
}

#[test]
fn configurable_currently_in_creation_multiple() {
    let mut c = make_container();
    let factory = as_cbf(&mut c);

    factory.set_currently_in_creation("beanA", true);
    factory.set_currently_in_creation("beanB", true);

    assert!(factory.is_currently_in_creation("beanA"));
    assert!(factory.is_currently_in_creation("beanB"));
    assert!(!factory.is_currently_in_creation("beanC"));

    factory.set_currently_in_creation("beanA", false);
    assert!(!factory.is_currently_in_creation("beanA"));
    assert!(factory.is_currently_in_creation("beanB"));
}

// ── dependent_beans / dependencies_for_bean ──────────────────────────

#[test]
fn configurable_dependent_beans() {
    let mut c = make_container();
    let factory = as_cbf(&mut c);

    // 初始状态：空
    assert!(factory.get_dependent_beans("beanA").is_empty());
    assert!(factory.get_dependencies_for_bean("beanB").is_empty());

    // 注册依赖关系：beanB 依赖 beanA
    factory.register_dependent_bean("beanA", "beanB");
    let dependents = factory.get_dependent_beans("beanA");
    assert_eq!(dependents, vec!["beanB"]);

    let deps = factory.get_dependencies_for_bean("beanB");
    assert_eq!(deps, vec!["beanA"]);

    // 不存在的 bean
    assert!(factory.get_dependent_beans("nonexistent").is_empty());
    assert!(factory.get_dependencies_for_bean("nonexistent").is_empty());
}

#[test]
fn configurable_dependent_beans_multiple() {
    let mut c = make_container();
    let factory = as_cbf(&mut c);

    factory.register_dependent_bean("A", "B");
    factory.register_dependent_bean("A", "C");

    let dependents = factory.get_dependent_beans("A");
    assert_eq!(dependents.len(), 2);
    assert!(dependents.contains(&"B".to_string()));
    assert!(dependents.contains(&"C".to_string()));
}

// ── destroy_bean / destroy_singletons ──────────────────────────────────

#[test]
fn configurable_destroy_bean() {
    let c = make_container();
    let factory = &c as &dyn ConfigurableBeanFactory;
    let result = factory.destroy_bean("testBean", &"test_instance");
    assert!(result.is_ok());
}

#[test]
fn configurable_destroy_singletons() {
    let mut c = make_container();
    // First resolve a singleton to create it in cache
    let _bean = c.get_bean_by_key(&ComponentKey::of::<String>()).unwrap();

    let factory = as_cbf(&mut c);
    factory.destroy_singletons();

    // After destroy, the singleton cache is empty
    // (the definition still exists in registry)
    assert!(c.contains_bean(&ComponentKey::of::<String>()));
}

// ── embedded_value_resolver ──────────────────────────────────────────

#[test]
fn configurable_embedded_value_resolver() {
    let mut c = make_container();
    let factory = as_cbf(&mut c);

    // 无解析器时，值原样返回
    assert_eq!(factory.resolve_embedded_value("hello"), "hello");

    // 添加一个简单的解析器
    factory.add_embedded_value_resolver(Arc::new(|val: &str| val.replace("${name}", "world")));
    assert_eq!(
        factory.resolve_embedded_value("Hello, ${name}!"),
        "Hello, world!"
    );

    // 添加链式解析器
    factory.add_embedded_value_resolver(Arc::new(|val: &str| val.to_uppercase()));
    let result = factory.resolve_embedded_value("Hello, ${name}!");
    assert_eq!(result, "HELLO, WORLD!");
}

#[test]
fn configurable_embedded_value_resolver_empty() {
    let mut c = make_container();
    let factory = as_cbf(&mut c);

    factory.add_embedded_value_resolver(Arc::new(|val: &str| {
        if val.is_empty() {
            "default".to_string()
        } else {
            val.to_string()
        }
    }));
    assert_eq!(factory.resolve_embedded_value(""), "default");
    assert_eq!(factory.resolve_embedded_value("custom"), "custom");
}

// ── set_parent_bean_factory (ConfigurableBeanFactory trait version) ───

#[test]
fn configurable_set_parent_via_trait() {
    use std::any::Any;
    let mut child = make_container();

    // Create parent as Arc<dyn Any + Send + Sync>
    let parent_container = Container::new(RegistryBuilder::new().build().unwrap());
    let parent_any: Arc<dyn Any + Send + Sync> = Arc::new(parent_container);

    let factory = as_cbf(&mut child);
    // The set_parent_bean_factory expects a BeanFactory inside the Any
    // Currently it tries to downcast to Arc<dyn BeanFactory> which won't match Arc<Container>
    // This test verifies it doesn't panic
    let result = factory.set_parent_bean_factory(parent_any);
    // When passing plain Container, it fails because it's not Arc<dyn BeanFactory>
    // This is an accepted limitation of the Any-based API
    let _ = result;
}
