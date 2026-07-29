use std::sync::Arc;
use vernal_beans::{
    BeanDefinitionRegistry, BeanFactory, ComponentDefinition, ComponentKey, Container, Qualifier,
    RegistryBuilder, hierarchical_bean_factory::HierarchicalBeanFactory,
};

fn as_reg(c: &Container) -> &dyn BeanDefinitionRegistry {
    c
}
fn as_reg_mut(c: &mut Container) -> &mut dyn BeanDefinitionRegistry {
    c
}
fn as_hier(c: &Container) -> &dyn HierarchicalBeanFactory {
    c
}

fn make_c() -> Container {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }));
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32));
    Container::new(b.build().unwrap())
}

#[test]
fn reg_contains() {
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    let mut c = make_c();
    let def = Box::new(RootBeanDefinition::new()) as Box<dyn vernal_beans::BeanDefinition>;
    as_reg_mut(&mut c)
        .register_bean_definition("dyn".to_string(), def)
        .unwrap();
    assert!(as_reg(&c).contains_bean_definition("dyn"));
}

#[test]
fn reg_names() {
    let c = make_c();
    assert!(as_reg(&c).bean_definition_names().len() >= 2);
}

#[test]
fn reg_remove_fail() {
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(as_reg_mut(&mut c).remove_bean_definition("x").is_err());
}

#[test]
fn reg_remove_ok() {
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    let def = Box::new(RootBeanDefinition::new()) as Box<dyn vernal_beans::BeanDefinition>;
    as_reg_mut(&mut c)
        .register_bean_definition("t".to_string(), def)
        .unwrap();
    assert!(as_reg_mut(&mut c).remove_bean_definition("t").is_ok());
}

#[test]
fn hier_set_parent() {
    let mut child = Container::new(RegistryBuilder::new().build().unwrap());
    child.set_parent_bean_factory(Arc::new(make_c()) as Arc<dyn BeanFactory>);
    assert!(as_hier(&child).parent_bean_factory().is_some());
}

#[test]
fn provider_patterns() {
    let c = make_c();
    let p = c
        .get_bean_provider_by_type_id(std::any::TypeId::of::<String>())
        .unwrap();
    p.if_available();
    let _ok = p.get();
    p.if_available();
    p.get_if_unique();
    p.stream();
}
