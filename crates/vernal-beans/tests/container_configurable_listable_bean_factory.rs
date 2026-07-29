//! Container 实现 ConfigurableListableBeanFactory trait 的集成测试。
//!
//! 对应 Spring 的 `ConfigurableListableBeanFactory` 语义。

use std::any::Any;
use std::sync::Arc;
use vernal_beans::{
    BeanFactory, ComponentDefinition, ComponentKey, Container, RegistryBuilder, Resolver,
    configurable_listable_bean_factory::ConfigurableListableBeanFactory,
};

fn make_container() -> Container {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }));
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32));
    Container::new(b.build().unwrap())
}

fn as_clbf(c: &mut Container) -> &mut dyn ConfigurableListableBeanFactory {
    c
}

// ── ignore_dependency_type / ignore_dependency_interface ────────────────

#[test]
fn configurable_listable_ignore_dependency_type() {
    let mut c = make_container();
    let factory = as_clbf(&mut c);

    // 初始：is_autowire_candidate 应该返回 true
    assert!(factory.is_autowire_candidate(std::any::type_name::<String>()));

    // 忽略 String 类型
    factory.ignore_dependency_type(std::any::TypeId::of::<String>());
    assert!(!factory.is_autowire_candidate(std::any::type_name::<String>()));

    // i32 仍然可用
    assert!(factory.is_autowire_candidate(std::any::type_name::<i32>()));
}

#[test]
fn configurable_listable_ignore_dependency_interface() {
    let mut c = make_container();
    let factory = as_clbf(&mut c);

    // ignore_dependency_interface 与 ignore_dependency_type 行为相同
    factory.ignore_dependency_interface(std::any::TypeId::of::<String>());
    assert!(!factory.is_autowire_candidate(std::any::type_name::<String>()));
}

#[test]
fn configurable_listable_is_autowire_candidate_unknown_bean() {
    let c = make_container();
    let factory = &c as &dyn ConfigurableListableBeanFactory;

    // 未知 bean 不是 autowire candidate
    assert!(!factory.is_autowire_candidate("NonExistentBean"));
}

// ── register_resolvable_dependency ───────────────────────────────────

#[test]
fn configurable_listable_register_resolvable_dependency() {
    let mut c = make_container();
    let factory = as_clbf(&mut c);

    // 注册一个可解析依赖
    factory.register_resolvable_dependency(
        std::any::TypeId::of::<String>(),
        Arc::new("resolved_value".to_string()) as Arc<dyn Any + Send + Sync>,
    );

    // 目前只是存储，不提供 getter
    // 验证没有 panic
}

// ── freeze_configuration / is_configuration_frozen ──────────────────

#[test]
fn configurable_listable_freeze_configuration() {
    let mut c = make_container();
    let factory = as_clbf(&mut c);

    // 初始：未冻结
    assert!(!factory.is_configuration_frozen());

    // 冻结
    factory.freeze_configuration();
    assert!(factory.is_configuration_frozen());

    // 再次冻结仍然处于冻结状态
    factory.freeze_configuration();
    assert!(factory.is_configuration_frozen());
}

#[test]
fn configurable_listable_freeze_new_container() {
    let mut c = make_container();
    let factory = as_clbf(&mut c);
    assert!(!factory.is_configuration_frozen());
}

// ── pre_instantiate_singletons ─────────────────────────────────────

#[test]
fn configurable_listable_pre_instantiate_singletons() {
    let c = make_container();
    let factory = &c as &dyn ConfigurableListableBeanFactory;

    // 预实例化所有 singleton
    let result = factory.pre_instantiate_singletons();
    assert!(result.is_ok());

    // 验证 singleton 都已创建（可通过 get_bean 访问）
    let bean = c
        .get_bean_by_type_id(std::any::TypeId::of::<String>())
        .unwrap();
    let s = (*bean).downcast_ref::<String>().unwrap();
    assert_eq!(s, "hello");
}

#[test]
fn configurable_listable_pre_instantiate_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let factory = &c as &dyn ConfigurableListableBeanFactory;

    let result = factory.pre_instantiate_singletons();
    assert!(result.is_ok());
}

#[test]
fn configurable_listable_pre_instantiate_with_transient_only() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::transient::<String, _>(|_| {
        "temp".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let factory = &c as &dyn ConfigurableListableBeanFactory;

    // 只有 transient bean，没有 singleton 可实例化
    let result = factory.pre_instantiate_singletons();
    assert!(result.is_ok());
}

#[test]
fn configurable_listable_pre_instantiate_multiple_singletons() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }));
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32));
    b.register(ComponentDefinition::transient::<f64, _>(|_| 3.14f64));
    let c = Container::new(b.build().unwrap());
    let factory = &c as &dyn ConfigurableListableBeanFactory;

    let result = factory.pre_instantiate_singletons();
    assert!(result.is_ok());
}

// ── is_autowire_candidate with dynamic_definitions ────────────────────

#[test]
fn configurable_listable_is_autowire_candidate_with_dynamic() {
    use vernal_beans::bean_definition::BeanDefinition;
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
    use vernal_beans::root_bean_definition::RootBeanDefinition;

    let mut c = make_container();
    let mut def = RootBeanDefinition::new();
    def.set_bean_class_name("dynamic.test.Bean");
    c.register_bean_definition("dynamicBean".to_string(), Box::new(def))
        .unwrap();

    let factory = &c as &dyn ConfigurableListableBeanFactory;
    assert!(factory.is_autowire_candidate("dynamicBean"));
}

#[test]
fn configurable_listable_is_autowire_candidate_deleted_dynamic() {
    use vernal_beans::bean_definition::BeanDefinition;
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
    use vernal_beans::root_bean_definition::RootBeanDefinition;

    let mut c = make_container();
    let mut def = RootBeanDefinition::new();
    def.set_bean_class_name("temp.Bean");
    c.register_bean_definition("tempBean".to_string(), Box::new(def))
        .unwrap();
    let _ = c.remove_bean_definition("tempBean");

    let factory = &c as &dyn ConfigurableListableBeanFactory;
    // deleted dynamic definitions should not be autowire candidates
    assert!(!factory.is_autowire_candidate("tempBean"));
}
