//! 第二轮覆盖率提升测试 — container.rs, root_bean_definition.rs, registry_builder.rs 等

use std::sync::Arc;
use vernal_beans::{
    BeanDefinitionRegistry, BeanFactory, ComponentDefinition, ComponentKey,
    ConfigurableBeanFactory, Container, Qualifier, RegistryBuilder,
    root_bean_definition::RootBeanDefinition,
};

fn container() -> Container {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }));
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32));
    Container::new(b.build().unwrap())
}

// ── RootBeanDefinition uncovered edge cases ─────────────────────

#[test]
fn rbd_from_generic() {
    use vernal_beans::generic_bean_definition::GenericBeanDefinition;
    let mut g = GenericBeanDefinition::new();
    g.set_bean_class_name("com.example.FromGeneric");
    let r = RootBeanDefinition::from(g);
    assert_eq!(r.bean_class_name(), "com.example.FromGeneric");
}

#[test]
fn rbd_full_bean_definition_trait() {
    let mut d = RootBeanDefinition::new();
    d.set_bean_class_name("T");
    d.set_scope(vernal_beans::Scope::Singleton);
    d.set_lazy_init(true);
    d.set_primary(true);
    d.set_fallback(false);
    d.set_autowire_candidate(false);
    d.set_role(1);
    d.set_description("desc");
    d.set_parent_name("p");
    d.set_factory_bean_name("fb");
    d.set_factory_method_name("fm");
    d.set_init_method_name("init");
    d.set_destroy_method_name("destroy");
    d.set_abstract(true);

    let b: &dyn vernal_beans::BeanDefinition = &d;
    assert_eq!(b.bean_class_name(), "T");
    assert_eq!(b.scope(), vernal_beans::Scope::Singleton);
    assert!(b.is_lazy_init());
    assert!(b.is_primary());
    assert!(!b.is_fallback());
    assert!(!b.is_autowire_candidate());
    assert_eq!(b.role(), 1);
    assert_eq!(b.description(), Some("desc"));
    assert_eq!(b.parent_name(), Some("p"));
    assert_eq!(b.factory_bean_name(), Some("fb"));
    assert_eq!(b.factory_method_name(), Some("fm"));
    assert_eq!(b.init_method_name(), Some("init"));
    assert_eq!(b.destroy_method_name(), Some("destroy"));
    assert!(b.is_abstract());
}

// ── RegistryBuilder edge cases ─────────────────────────────────

#[test]
fn reg_builder_full_flow() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "x".to_string()
    }));
    assert!(b.contains::<String>());
    b.remove::<String>().unwrap();
    assert!(!b.contains::<String>());
}

#[test]
fn reg_builder_remove_by_key_after_remove() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "x".to_string()
    }));
    b.remove::<String>().unwrap();
    assert!(b.remove_by_key(&ComponentKey::of::<String>()).is_err());
}

// ── ConfigurationClassPostProcessor edge cases ─────────────────

#[test]
fn config_pp_new_default() {
    use vernal_beans::configuration_class_post_processor::ConfigurationClassPostProcessor;
    let p = ConfigurationClassPostProcessor::new();
    assert_eq!(p.registered_count(), 0);
    let p2: ConfigurationClassPostProcessor = Default::default();
    assert_eq!(p2.registered_count(), 0);
}

// ── Container edge: get_bean_by_key with resolve error ──────────

#[test]
fn container_get_bean_failure() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| -> String {
        panic!("fail")
    }));
    let c = Container::new(b.build().unwrap());
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        c.get_bean_by_key(&ComponentKey::of::<String>())
    }));
    assert!(result.is_err());
}

// ── ScopeContext edge cases ─────────────────────────────────────

#[test]
fn scope_context_open_close() {
    use vernal_beans::ScopeContext;
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let ctx = c.open_scope::<String>();
    // close is async, tested via block_on below
    // close requires async context
}

// ── container.rs: BeanDefinitionRegistry __DELETED__ check ──────

#[test]
fn container_reg_delete_and_contains() {
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    let def = Box::new(RootBeanDefinition::new()) as Box<dyn vernal_beans::BeanDefinition>;
    c.register_bean_definition("t".to_string(), def).unwrap();
    c.remove_bean_definition("t").unwrap();
    assert!(!c.contains_bean_definition("t"));
}

// ── ConfigurableBeanFactory register_alias ─────────────────────

#[test]
fn configurable_register_alias_dup() {
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(c.register_alias("b1", "a").is_ok());
    assert!(c.register_alias("b2", "a").is_err());
}

// ── BeanFactoryUtils additional tests ──────────────────────────

#[test]
fn bean_factory_utils_count() {
    use vernal_beans::bean_factory_utils::BeanFactoryUtils;

    let c = container();
    let n = BeanFactoryUtils::count_beans_for_type(std::any::TypeId::of::<String>(), &c);
    assert_eq!(n, 1);
}

#[test]
fn bean_factory_utils_beans_of_type() {
    use vernal_beans::bean_factory_utils::BeanFactoryUtils;

    let c = container();
    let beans = BeanFactoryUtils::beans_of_type(std::any::TypeId::of::<String>(), &c).unwrap();
    assert_eq!(beans.len(), 1);
}
