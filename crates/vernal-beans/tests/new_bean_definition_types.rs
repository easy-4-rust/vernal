//! 新创建的 BeanDefinition 相关类型的覆盖测试。

use std::sync::Arc;
use vernal_beans::{
    BeanDefinition, BeanDefinitionDefaults, BeanDefinitionHolder, ChildBeanDefinition,
    RootBeanDefinition, Scope,
};

// ── BeanDefinitionDefaults ──────────────────────────────────────────

#[test]
fn bean_definition_defaults_new() {
    let d = BeanDefinitionDefaults::new();
    assert_eq!(d.scope(), Scope::Singleton);
    assert!(!d.is_lazy_init());
    assert_eq!(d.autowire_mode(), vernal_beans::Autowire::No);
    assert!(d.init_method_name().is_none());
    assert!(d.destroy_method_name().is_none());
}

#[test]
fn bean_definition_defaults_setters() {
    let mut d = BeanDefinitionDefaults::new();
    d.set_scope(Scope::Transient);
    assert_eq!(d.scope(), Scope::Transient);

    d.set_lazy_init(true);
    assert!(d.is_lazy_init());

    d.set_autowire_mode(vernal_beans::Autowire::ByType);
    assert_eq!(d.autowire_mode(), vernal_beans::Autowire::ByType);

    d.set_init_method_name("init");
    assert_eq!(d.init_method_name(), Some("init"));

    d.set_destroy_method_name("destroy");
    assert_eq!(d.destroy_method_name(), Some("destroy"));
}

#[test]
fn bean_definition_defaults_default() {
    let d: BeanDefinitionDefaults = Default::default();
    assert_eq!(d.scope(), Scope::Singleton);
}

// ── BeanDefinitionHolder ────────────────────────────────────────────

#[test]
fn bean_definition_holder_new() {
    let def = Arc::new(RootBeanDefinition::new()) as Arc<dyn BeanDefinition>;
    let holder = BeanDefinitionHolder::new(def, "testBean");
    assert_eq!(holder.bean_name(), "testBean");
    assert!(holder.aliases().is_empty());
}

#[test]
fn bean_definition_holder_with_aliases() {
    let def = Arc::new(RootBeanDefinition::new()) as Arc<dyn BeanDefinition>;
    let holder = BeanDefinitionHolder::with_aliases(
        def,
        "primaryBean",
        vec!["alias1".to_string(), "alias2".to_string()],
    );
    assert_eq!(holder.aliases().len(), 2);
}

#[test]
fn bean_definition_holder_add_alias() {
    let def = Arc::new(RootBeanDefinition::new()) as Arc<dyn BeanDefinition>;
    let mut holder = BeanDefinitionHolder::new(def, "test");
    holder.add_alias("myAlias");
    assert_eq!(holder.aliases(), &["myAlias"]);
}

#[test]
fn bean_definition_holder_bean_definition() {
    use vernal_beans::bean_definition::BeanDefinition;
    let def = Arc::new(RootBeanDefinition::new()) as Arc<dyn BeanDefinition>;
    let holder = BeanDefinitionHolder::new(def, "test");
    assert_eq!(holder.bean_definition().bean_class_name(), "unknown");
}

#[test]
fn bean_definition_holder_short_name() {
    let def = Arc::new(RootBeanDefinition::new()) as Arc<dyn BeanDefinition>;
    let holder = BeanDefinitionHolder::new(def, "my::nested::Module");
    assert_eq!(holder.short_name(), "Module");
}

#[test]
fn bean_definition_holder_short_name_simple() {
    let def = Arc::new(RootBeanDefinition::new()) as Arc<dyn BeanDefinition>;
    let holder = BeanDefinitionHolder::new(def, "SimpleName");
    assert_eq!(holder.short_name(), "SimpleName");
}

// ── ChildBeanDefinition ─────────────────────────────────────────────

#[test]
fn child_bean_definition_new() {
    let child = ChildBeanDefinition::new("parentBean");
    assert_eq!(child.parent_name(), "parentBean");
    assert_eq!(child.scope(), Scope::Singleton);
    assert!(!child.is_lazy_init());
    assert!(!child.is_primary());
    assert!(child.bean_class_name().is_none());
}

#[test]
fn child_bean_definition_setters() {
    let mut child = ChildBeanDefinition::new("parent");
    child.set_bean_class_name("com.example.ChildClass");
    assert_eq!(child.bean_class_name(), Some("com.example.ChildClass"));

    child.set_scope(Scope::Transient);
    assert_eq!(child.scope(), Scope::Transient);

    child.set_lazy_init(true);
    assert!(child.is_lazy_init());

    child.set_primary(true);
    assert!(child.is_primary());
}

#[test]
fn child_bean_definition_bean_definition_trait() {
    let mut child = ChildBeanDefinition::new("parent");
    child.set_bean_class_name("com.example.Child");

    // Call through BeanDefinition trait explicitly
    use vernal_beans::bean_definition::BeanDefinition;
    let def: &dyn BeanDefinition = &child;
    assert_eq!(def.bean_class_name(), "com.example.Child");
    assert_eq!(def.scope(), Scope::Singleton);
    assert!(!def.is_lazy_init());
    assert!(!def.is_primary());
    assert_eq!(def.parent_name(), Some("parent"));
}

#[test]
fn child_bean_definition_merge_into() {
    let root = RootBeanDefinition::new();
    let mut child = ChildBeanDefinition::new("parent");
    child.set_bean_class_name("com.example.Merged");
    child.set_scope(Scope::Transient);
    child.set_primary(true);

    let merged = child.merge_into(&root);
    assert_eq!(merged.bean_class_name(), "com.example.Merged");
    assert_eq!(merged.scope(), Scope::Transient);
    assert!(merged.is_primary());
}
