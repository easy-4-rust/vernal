//! 针对性覆盖测试 — bean_definition_utils.rs + component_key/trait_key

use std::sync::Arc;
use vernal_beans::{BeanDefinition, ComponentKey, Container, RegistryBuilder, Scope};

// ── generate_bean_name with conflict ────────────────────────────────────

#[test]
fn generate_bean_name_conflict() {
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
    use vernal_beans::bean_definition_utils;

    let mut c = Container::new(RegistryBuilder::new().build().unwrap());

    let name = "com.example.Test".to_string();
    c.register_bean_definition(name.clone(), Box::new(CloneBeanDef(name.clone())))
        .unwrap();

    let generated = bean_definition_utils::generate_bean_name(Some("com.example.Test"), &c);
    assert!(generated.contains("com.example.Test"));
}

#[derive(Clone, Debug)]
struct CloneBeanDef(String);

impl BeanDefinition for CloneBeanDef {
    fn bean_name(&self) -> &ComponentKey {
        unimplemented!("not used")
    }
    fn bean_class_name(&self) -> &str {
        &self.0
    }
    fn scope(&self) -> Scope {
        Scope::Singleton
    }
    fn is_lazy_init(&self) -> bool {
        false
    }
    fn is_primary(&self) -> bool {
        false
    }
}

#[test]
fn generate_bean_name_anonymous() {
    use vernal_beans::bean_definition_utils;

    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let name = bean_definition_utils::generate_bean_name(None, &c);
    assert_eq!(name, "anonymous");
}

// ── ComponentKey display & qualifier ────────────────────────────────────

#[test]
fn component_key_display_and_methods() {
    let key = ComponentKey::of::<String>();
    let s = format!("{}", key);
    assert!(s.contains("String"));
    assert_eq!(key.type_name(), std::any::type_name::<String>());
    assert!(key.qualifier().is_none());
}

#[test]
fn component_key_with_qualifier() {
    use vernal_beans::Qualifier;
    let q = Qualifier::new("primary").unwrap();
    let key = ComponentKey::qualified::<String>(q.clone());
    assert_eq!(key.qualifier(), Some(&q));
    let s = format!("{}", key);
    assert!(s.contains("primary") || s.contains("String"));
}

// ── TraitKey display ────────────────────────────────────────────────────

#[test]
fn trait_key_display() {
    use vernal_beans::TraitKey;
    let tk = TraitKey::of::<dyn std::fmt::Debug>();
    let s = format!("{}", tk);
    // Should contain the type name
    assert!(!s.is_empty());
}

// ── BeanFactoryUtils more edge cases ────────────────────────────────────

#[test]
fn bean_factory_utils_more() {
    use vernal_beans::bean_factory_utils::BeanFactoryUtils;

    // transformed_bean_name with no prefix
    assert_eq!(BeanFactoryUtils::transformed_bean_name("normal"), "normal");
    // empty string
    assert_eq!(BeanFactoryUtils::transformed_bean_name(""), "");
    // just & prefix
    assert_eq!(BeanFactoryUtils::transformed_bean_name("&"), "");
}
