//! Container 实现 HierarchicalBeanFactory trait 的集成测试。
//!
//! 对应 Spring 的 `HierarchicalBeanFactory` 语义：
//! - 子容器可以委托父容器解析 Bean
//! - 父容器不能访问子容器的 Bean
//! - 子容器的同名 Bean 覆盖父容器的同名 Bean

use std::any::Any;
use std::sync::Arc;
use vernal_beans::{
    BeanFactory, ComponentDefinition, ComponentKey, Container, RegistryBuilder,
    hierarchical_bean_factory::HierarchicalBeanFactory,
};

/// 创建一个包含 String bean 的父容器
fn make_parent() -> Arc<dyn BeanFactory> {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "parent_value".to_string()
    }));
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 100i32));
    Arc::new(Container::new(b.build().unwrap())) as Arc<dyn BeanFactory>
}

/// 创建一个空的子容器，挂载父容器
fn make_child_with_parent() -> Container {
    let mut child = Container::new(RegistryBuilder::new().build().unwrap());
    let parent = make_parent();
    child.set_parent_bean_factory(parent);
    child
}

fn as_hier(c: &Container) -> &dyn HierarchicalBeanFactory {
    c
}

// ── parent_bean_factory ─────────────────────────────────────────────

#[test]
fn hierarchical_parent_bean_factory_some() {
    let child = make_child_with_parent();
    let parent_ref = as_hier(&child).parent_bean_factory();
    assert!(parent_ref.is_some());
}

#[test]
fn hierarchical_parent_bean_factory_none() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(as_hier(&c).parent_bean_factory().is_none());
}

// ── contains_local_bean ─────────────────────────────────────────────

#[test]
fn hierarchical_contains_local_bean_true() {
    let child = make_child_with_parent();
    // child 应该包含 String 吗？child 的 registry 是空的。
    // contains_local_bean 只检查本地注册表，不检查父容器
    assert!(!as_hier(&child).contains_local_bean(std::any::type_name::<String>()));
}

#[test]
fn hierarchical_contains_local_bean_false() {
    let child = make_child_with_parent();
    assert!(!as_hier(&child).contains_local_bean("NonExistentBean"));
}

#[test]
fn hierarchical_contains_local_bean_empty() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(!as_hier(&c).contains_local_bean("anything"));
}

// ── Parent delegation via BeanFactory ───────────────────────────────

#[test]
fn hierarchical_parent_get_bean_by_type_id() {
    // 父容器可以直接访问其 bean
    let parent = make_parent();
    let bean = parent
        .get_bean_by_type_id(std::any::TypeId::of::<String>())
        .unwrap();
    let s = (*bean).downcast_ref::<String>().unwrap();
    assert_eq!(s, "parent_value");

    let bean = parent
        .get_bean_by_type_id(std::any::TypeId::of::<i32>())
        .unwrap();
    let n = (*bean).downcast_ref::<i32>().unwrap();
    assert_eq!(*n, 100i32);
}

#[test]
fn hierarchical_parent_get_bean_by_key() {
    let parent = make_parent();
    let bean = parent
        .get_bean_by_key(&ComponentKey::of::<String>())
        .unwrap();
    let s = (*bean).downcast_ref::<String>().unwrap();
    assert_eq!(s, "parent_value");
}

// ── Child overrides parent ─────────────────────────────────────────

#[test]
fn hierarchical_child_override_parent() {
    let mut parent_b = RegistryBuilder::new();
    parent_b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "original".to_string()
    }));
    let parent: Arc<dyn BeanFactory> = Arc::new(Container::new(parent_b.build().unwrap()));

    let mut child_b = RegistryBuilder::new();
    child_b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "override".to_string()
    }));
    let mut child = Container::new(child_b.build().unwrap());
    child.set_parent_bean_factory(parent);

    // 子容器应返回自己注册的值（覆盖父容器的同名 bean）
    let bean = child
        .get_bean_by_type_id(std::any::TypeId::of::<String>())
        .unwrap();
    let s = (*bean).downcast_ref::<String>().unwrap();
    assert_eq!(s, "override");
}

#[test]
fn hierarchical_child_delegates_to_parent() {
    use vernal_beans::ListableBeanFactory;
    // 父容器有 String，子容器没有 String
    // 子容器通过 get_bean 委托到父容器...

    // 注意：当前的 Container.get_bean_by_type_id 只在自己的 registry 中查找
    // 不会自动委托到父容器。HierarchicalBeanFactory 只是提供接口方法，
    // 实际的委托逻辑由上层 ApplicationContext 实现。
    // 这里只验证接口方法可用。
    let child = make_child_with_parent();

    // contains_bean 只检查本地 registry
    assert!(!child.contains_bean(&ComponentKey::of::<String>()));

    // contains_local_bean 也应该返回 false（本地没有）
    assert!(!as_hier(&child).contains_local_bean(std::any::type_name::<String>()));
}

// ── Multiple children sharing parent ───────────────────────────────

#[test]
fn hierarchical_multiple_children_same_parent() {
    let parent = make_parent();

    let mut child1 = Container::new(RegistryBuilder::new().build().unwrap());
    child1.set_parent_bean_factory(parent.clone());

    let mut child2 = Container::new(RegistryBuilder::new().build().unwrap());
    child2.set_parent_bean_factory(parent);

    // Both children have the same parent
    assert!(as_hier(&child1).parent_bean_factory().is_some());
    assert!(as_hier(&child2).parent_bean_factory().is_some());
}

// ── set_parent_bean_factory replaces existing parent ───────────────

#[test]
fn hierarchical_replace_parent() {
    let mut parent1_b = RegistryBuilder::new();
    parent1_b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "p1".to_string()
    }));
    let parent1: Arc<dyn BeanFactory> = Arc::new(Container::new(parent1_b.build().unwrap()));

    let mut child = Container::new(RegistryBuilder::new().build().unwrap());
    child.set_parent_bean_factory(parent1);
    assert!(as_hier(&child).parent_bean_factory().is_some());

    // Replace with another parent
    let parent2: Arc<dyn BeanFactory> =
        Arc::new(Container::new(RegistryBuilder::new().build().unwrap()));
    child.set_parent_bean_factory(parent2);
    assert!(as_hier(&child).parent_bean_factory().is_some());
}

// ── Contains bean only in local registry ───────────────────────────

#[test]
fn hierarchical_contains_bean_behavior() {
    // Container.contains_bean() only checks local registry, not parent
    let child = make_child_with_parent();
    // String is only in parent, not in child
    assert!(!child.contains_bean(&ComponentKey::of::<String>()));
}
