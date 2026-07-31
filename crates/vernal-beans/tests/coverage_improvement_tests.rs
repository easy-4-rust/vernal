/// Comprehensive coverage improvement tests for vernal-beans.
///
/// Targets files with the most uncovered lines to maximize code coverage.
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use vernal_beans::{
    ComponentDefinition, Container, Qualifier, RegistryBuilder,
    Scope, TraitBinding, PropertyEditor,
};

// ═══════════════════════════════════════════════════════════════════════════════
// 1. container.rs — Resolve variants, scope operations, bean lifecycle
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_trait_single_binding() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
    b.bind(binding).unwrap();
    let c = Container::new(b.build().unwrap());
    let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> = c.resolve_trait();
    assert!(result.is_ok());
}

#[test]
fn container_resolve_trait_not_found() {
    let b = RegistryBuilder::new();
    let c = Container::new(b.build().unwrap());
    let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> = c.resolve_trait();
    assert!(result.is_err());
}

#[test]
fn container_resolve_trait_in_wrong_owner() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
    b.bind(binding).unwrap();
    let c = Container::new(b.build().unwrap());
    let other = Container::new(RegistryBuilder::new().build().unwrap());
    let scope = other.open_scope::<String>();
    let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> = c.resolve_trait_in(&scope);
    assert!(result.is_err());
}

#[test]
fn container_resolve_qualified_trait() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    let q = Qualifier::new("primary").unwrap();
    let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)
        .qualified(q.clone());
    b.bind(binding).unwrap();
    let c = Container::new(b.build().unwrap());
    let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> =
        c.resolve_qualified_trait(&q);
    assert!(result.is_ok());
}

#[test]
fn container_resolve_qualified_trait_in_wrong_owner() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    let q = Qualifier::new("q").unwrap();
    let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)
        .qualified(q.clone());
    b.bind(binding).unwrap();
    let c = Container::new(b.build().unwrap());
    let other = Container::new(RegistryBuilder::new().build().unwrap());
    let scope = other.open_scope::<String>();
    let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> =
        c.resolve_qualified_trait_in(&q, &scope);
    assert!(result.is_err());
}

#[test]
fn container_resolve_all_traits_empty() {
    let b = RegistryBuilder::new();
    let c = Container::new(b.build().unwrap());
    let result: Result<Vec<Arc<dyn std::fmt::Display + Send + Sync>>, _> = c.resolve_all_traits();
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn container_resolve_all_traits_multiple() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32));
    let binding1 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
    let binding2 = TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
    b.bind(binding1).unwrap();
    b.bind(binding2).unwrap();
    let c = Container::new(b.build().unwrap());
    let result: Result<Vec<Arc<dyn std::fmt::Display + Send + Sync>>, _> = c.resolve_all_traits();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 2);
}

#[test]
fn container_resolve_all_traits_in_wrong_owner() {
    let b = RegistryBuilder::new();
    let c = Container::new(b.build().unwrap());
    let other = Container::new(RegistryBuilder::new().build().unwrap());
    let scope = other.open_scope::<String>();
    let result: Result<Vec<Arc<dyn std::fmt::Display + Send + Sync>>, _> =
        c.resolve_all_traits_in(&scope);
    assert!(result.is_err());
}

#[test]
fn container_resolve_in_correct_owner() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    let c = Container::new(b.build().unwrap());
    let scope = c.open_scope::<String>();
    let result: Result<Arc<String>, _> = c.resolve_in(&scope);
    assert!(result.is_ok());
}

#[test]
fn container_transient_tracks_instances() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::transient::<i32, _>(|_| 42i32));
    let c = Container::new(b.build().unwrap());
    let _: Arc<i32> = c.resolve().unwrap();
    let _tracker = c.transient_tracker();
}

#[test]
fn container_select_definition_ambiguous() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
            .qualified(Qualifier::new("q1").unwrap()),
    );
    let c = Container::new(b.build().unwrap());
    let result: Result<Arc<String>, _> = c.resolve();
    assert!(result.is_err());
}

#[test]
fn container_warm_up_empty_registry() {
    let b = RegistryBuilder::new();
    let c = Container::new(b.build().unwrap());
    c.warm_up().unwrap();
}

#[test]
fn container_warm_up_with_transient() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::transient::<String, _>(|_| "t".to_string()));
    let c = Container::new(b.build().unwrap());
    c.warm_up().unwrap();
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. registry_builder.rs — Builder methods, error cases
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn registry_builder_register_all_with_external_duplicate() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::shared_value(42i32)).unwrap();
    let result = b.register_all(vec![ComponentDefinition::shared_value(99i32)]);
    assert!(result.is_err());
    assert_eq!(b.len(), 1);
}

#[test]
fn registry_builder_register_bundle() {
    let mut b = RegistryBuilder::new();
    let defs = vec![ComponentDefinition::shared_value(42i32)];
    let bindings = vec![
        TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>),
    ];
    b.register_bundle(defs, bindings).unwrap();
    assert_eq!(b.len(), 1);
}

#[test]
fn registry_builder_register_bundle_atomic_failure() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::shared_value(42i32)).unwrap();
    let defs = vec![ComponentDefinition::shared_value(99i32)];
    let bindings = vec![];
    let result = b.register_bundle(defs, bindings);
    assert!(result.is_err());
    assert_eq!(b.len(), 1);
}

#[test]
fn registry_builder_bind_all() {
    let mut b = RegistryBuilder::new();
    let bindings = vec![
        TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>),
        TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>),
    ];
    b.bind_all(bindings).unwrap();
}

#[test]
fn registry_builder_bind_duplicate_exact() {
    let mut b = RegistryBuilder::new();
    let binding = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
    b.bind(binding).unwrap();
    let binding2 = TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
    let result = b.bind(binding2);
    assert!(result.is_err());
}

#[test]
fn registry_builder_bind_duplicate_qualified() {
    let mut b = RegistryBuilder::new();
    let q = Qualifier::new("myqualifier").unwrap();
    let binding1 =
        TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)
            .qualified(q.clone());
    let binding2 =
        TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>)
            .qualified(q.clone());
    b.bind(binding1).unwrap();
    let result = b.bind(binding2);
    assert!(result.is_err());
}

#[test]
fn registry_builder_bind_multiple_primary_fails() {
    let mut b = RegistryBuilder::new();
    let binding1 =
        TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>).primary();
    let binding2 =
        TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>).primary();
    b.bind(binding1).unwrap();
    let result = b.bind(binding2);
    assert!(result.is_err());
}

#[test]
fn registry_builder_bean_definition_registry_trait() {
    use vernal_beans::BeanDefinitionRegistry;
    let mut b = RegistryBuilder::new();
    // RegistryBuilder::register_bean_definition is a no-op stub,
    // so contains/bean_definition_count reflect the ComponentDefinition registry
    b.register(ComponentDefinition::shared_value(42i32)).unwrap();
    assert!(b.contains_bean_definition("i32"));
    assert_eq!(b.bean_definition_count(), 1);
    let names = b.bean_definition_names();
    assert!(names.contains(&"i32".to_string()));
}

#[test]
fn registry_builder_remove_bean_definition_by_type_name() {
    use vernal_beans::BeanDefinitionRegistry;
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::shared_value(42i32)).unwrap();
    // remove_bean_definition looks up by type_name
    let removed = b.remove_bean_definition("i32").unwrap();
    assert_eq!(removed.bean_class_name(), "i32");
}

#[test]
fn registry_builder_remove_bean_definition_not_found() {
    use vernal_beans::BeanDefinitionRegistry;
    let mut b = RegistryBuilder::new();
    let result = b.remove_bean_definition("nonexistent");
    assert!(result.is_err());
}

#[test]
fn registry_builder_get_bean_definition_returns_none() {
    use vernal_beans::BeanDefinitionRegistry;
    let b = RegistryBuilder::new();
    assert!(b.get_bean_definition("any").is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. factory/parsing/component_definition.rs — Component definition methods
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn component_definition_shared_value() {
    let def = ComponentDefinition::shared_value(42i32);
    assert_eq!(def.scope(), Scope::Singleton);
    assert!(def.dependencies().is_empty());
}

#[test]
fn component_definition_shared_arc() {
    let val = Arc::new(String::from("hello"));
    let def = ComponentDefinition::shared_arc(val);
    assert_eq!(def.scope(), Scope::Singleton);
}

#[test]
fn component_definition_try_singleton() {
    let def = ComponentDefinition::try_singleton::<String, _>(|_| Ok("hello".to_string()));
    assert_eq!(def.scope(), Scope::Singleton);
}

#[test]
fn component_definition_try_transient() {
    let def = ComponentDefinition::try_transient::<String, _>(|_| Ok("hello".to_string()));
    assert_eq!(def.scope(), Scope::Transient);
}

#[test]
fn component_definition_scoped() {
    struct MyScope;
    let def = ComponentDefinition::scoped::<String, MyScope, _>(|_| "hello".to_string());
    assert!(matches!(def.scope(), Scope::Custom(_)));
}

#[test]
fn component_definition_try_scoped() {
    struct MyScope;
    let def =
        ComponentDefinition::try_scoped::<String, MyScope, _>(|_| Ok("hello".to_string()));
    assert!(matches!(def.scope(), Scope::Custom(_)));
}

#[test]
fn component_definition_with_init_order() {
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
        .with_init_order(100);
    assert_eq!(def.init_order(), 100);
}

#[test]
fn component_definition_qualified() {
    let q = Qualifier::new("primary").unwrap();
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
        .qualified(q);
    assert!(def.key().qualifier().is_some());
}

#[test]
fn component_definition_depends_on() {
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
        .depends_on::<i32>();
    assert_eq!(def.dependencies().len(), 1);
}

#[test]
fn component_definition_depends_on_optional() {
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
        .depends_on_optional::<i32>();
    assert_eq!(def.dependencies().len(), 1);
}

#[test]
fn component_definition_depends_on_provider() {
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
        .depends_on_provider::<i32>();
    assert_eq!(def.dependencies().len(), 1);
}

#[test]
fn component_definition_depends_on_optional_provider() {
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
        .depends_on_optional_provider::<i32>();
    assert_eq!(def.dependencies().len(), 1);
}

#[test]
fn component_definition_depends_on_qualified() {
    let q = Qualifier::new("q").unwrap();
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
        .depends_on_qualified::<i32>(q);
    assert_eq!(def.dependencies().len(), 1);
}

#[test]
fn component_definition_depends_on_optional_qualified() {
    let q = Qualifier::new("q").unwrap();
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
        .depends_on_optional_qualified::<i32>(q);
    assert_eq!(def.dependencies().len(), 1);
}

#[test]
fn component_definition_depends_on_qualified_provider() {
    let q = Qualifier::new("q").unwrap();
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
        .depends_on_qualified_provider::<i32>(q);
    assert_eq!(def.dependencies().len(), 1);
}

#[test]
fn component_definition_depends_on_optional_qualified_provider() {
    let q = Qualifier::new("q").unwrap();
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
        .depends_on_optional_qualified_provider::<i32>(q);
    assert_eq!(def.dependencies().len(), 1);
}

#[test]
fn component_definition_depends_on_trait() {
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
        .depends_on_trait::<dyn std::fmt::Display + Send + Sync>();
    assert_eq!(def.dependencies().len(), 1);
}

#[test]
fn component_definition_depends_on_optional_trait() {
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
        .depends_on_optional_trait::<dyn std::fmt::Display + Send + Sync>();
    assert_eq!(def.dependencies().len(), 1);
}

#[test]
fn component_definition_depends_on_qualified_trait() {
    let q = Qualifier::new("q").unwrap();
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
        .depends_on_qualified_trait::<dyn std::fmt::Display + Send + Sync>(q);
    assert_eq!(def.dependencies().len(), 1);
}

#[test]
fn component_definition_depends_on_optional_qualified_trait() {
    let q = Qualifier::new("q").unwrap();
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
        .depends_on_optional_qualified_trait::<dyn std::fmt::Display + Send + Sync>(q);
    assert_eq!(def.dependencies().len(), 1);
}

#[test]
fn component_definition_depends_on_trait_provider() {
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
        .depends_on_trait_provider::<dyn std::fmt::Display + Send + Sync>();
    assert_eq!(def.dependencies().len(), 1);
}

#[test]
fn component_definition_depends_on_optional_trait_provider() {
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
        .depends_on_optional_trait_provider::<dyn std::fmt::Display + Send + Sync>();
    assert_eq!(def.dependencies().len(), 1);
}

#[test]
fn component_definition_depends_on_qualified_trait_provider() {
    let q = Qualifier::new("q").unwrap();
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
        .depends_on_qualified_trait_provider::<dyn std::fmt::Display + Send + Sync>(q);
    assert_eq!(def.dependencies().len(), 1);
}

#[test]
fn component_definition_depends_on_optional_qualified_trait_provider() {
    let q = Qualifier::new("q").unwrap();
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
        .depends_on_optional_qualified_trait_provider::<dyn std::fmt::Display + Send + Sync>(q);
    assert_eq!(def.dependencies().len(), 1);
}

#[test]
fn component_definition_depends_on_all_traits() {
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())
        .depends_on_all_traits::<dyn std::fmt::Display + Send + Sync>();
    assert_eq!(def.dependencies().len(), 1);
}

#[test]
fn component_definition_debug_format() {
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string());
    let debug = format!("{:?}", def);
    assert!(debug.contains("ComponentDefinition"));
}

#[test]
fn component_definition_ref() {
    use vernal_beans::factory::parsing::component_definition::ComponentDefinitionRef;
    let r = ComponentDefinitionRef::new("myBean", Some("MyClass".to_string()));
    assert_eq!(r.bean_name(), "myBean");
    assert_eq!(r.bean_class_name(), Some("MyClass"));
}

#[test]
fn component_definition_ref_no_class() {
    use vernal_beans::factory::parsing::component_definition::ComponentDefinitionRef;
    let r = ComponentDefinitionRef::new("myBean", None);
    assert_eq!(r.bean_name(), "myBean");
    assert!(r.bean_class_name().is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. factory/config/instantiation_aware_bean_post_processor.rs
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn instantiation_aware_post_processor_set_continue_injection() {
    use vernal_beans::factory::config::instantiation_aware_bean_post_processor::*;
    let mut p = SimpleInstantiationAwareBeanPostProcessor::new();
    p.set_continue_injection(false);
    let bean = Arc::new(String::from("test"));
    let result = p
        .post_process_after_instantiation(bean.as_ref(), "myBean")
        .unwrap();
    assert!(!result);
}

#[test]
fn instantiation_aware_post_processor_before_initialization() {
    use vernal_beans::factory::config::instantiation_aware_bean_post_processor::*;
    let p = SimpleInstantiationAwareBeanPostProcessor::new();
    let bean = Arc::new(String::from("test"));
    let result = p
        .post_process_before_initialization(bean, "myBean")
        .unwrap();
    assert!(result.is_some());
}

#[test]
fn instantiation_aware_post_processor_after_initialization() {
    use vernal_beans::factory::config::instantiation_aware_bean_post_processor::*;
    let p = SimpleInstantiationAwareBeanPostProcessor::new();
    let bean = Arc::new(String::from("test"));
    let result = p
        .post_process_after_initialization(bean, "myBean")
        .unwrap();
    assert!(result.is_some());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. charset_property_editor.rs
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn charset_editor_default() {
    let editor = vernal_beans::charset_property_editor::CharsetPropertyEditor::default();
    assert!(editor.get_as_text().is_none());
}

#[test]
fn charset_editor_set_and_get_text() {
    let mut editor = vernal_beans::charset_property_editor::CharsetPropertyEditor::new();
    editor.set_as_text("UTF-8").unwrap();
    assert_eq!(editor.get_as_text(), Some("UTF-8".to_string()));
}

#[test]
fn charset_editor_empty_text() {
    let mut editor = vernal_beans::charset_property_editor::CharsetPropertyEditor::new();
    editor.set_as_text("UTF-8").unwrap();
    editor.set_as_text("").unwrap();
    assert!(editor.get_as_text().is_none());
}

#[test]
fn charset_editor_whitespace_only() {
    let mut editor = vernal_beans::charset_property_editor::CharsetPropertyEditor::new();
    editor.set_as_text("   ").unwrap();
    assert!(editor.get_as_text().is_none());
}

#[test]
fn charset_editor_trimmed() {
    let mut editor = vernal_beans::charset_property_editor::CharsetPropertyEditor::new();
    editor.set_as_text("  UTF-8  ").unwrap();
    assert_eq!(editor.get_as_text(), Some("UTF-8".to_string()));
}

#[test]
fn charset_editor_set_value() {
    let mut editor = vernal_beans::charset_property_editor::CharsetPropertyEditor::new();
    let val: Arc<dyn Any + Send + Sync> = Arc::new(String::from("ASCII"));
    editor.set_value(val);
    assert_eq!(editor.get_as_text(), Some("ASCII".to_string()));
}

#[test]
fn charset_editor_set_value_wrong_type() {
    let mut editor = vernal_beans::charset_property_editor::CharsetPropertyEditor::new();
    let val: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    editor.set_value(val);
    assert!(editor.get_as_text().is_none());
}

#[test]
fn charset_editor_get_value() {
    let mut editor = vernal_beans::charset_property_editor::CharsetPropertyEditor::new();
    assert!(editor.get_value().is_none());
    editor.set_as_text("UTF-8").unwrap();
    assert!(editor.get_value().is_some());
}

#[test]
fn charset_editor_target_type() {
    let editor = vernal_beans::charset_property_editor::CharsetPropertyEditor::new();
    assert_eq!(editor.target_type(), TypeId::of::<String>());
}

#[test]
fn charset_editor_get_value_type() {
    let editor = vernal_beans::charset_property_editor::CharsetPropertyEditor::new();
    assert_eq!(editor.get_value_type(), TypeId::of::<String>());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. resolver.rs — Resolver methods
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn resolver_resolve_undeclared_dependency() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|resolver| {
        let result: Result<Arc<i32>, _> = resolver.resolve();
        assert!(result.is_err());
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let val: Arc<String> = c.resolve().unwrap();
    assert_eq!(*val, "hello");
}

#[test]
fn resolver_resolve_optional_undeclared() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|resolver| {
        let result: Result<Option<Arc<i32>>, _> = resolver.resolve_optional();
        assert!(result.is_err());
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let _: Arc<String> = c.resolve().unwrap();
}

#[test]
fn resolver_component_returns_key() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|resolver| {
        let _key = resolver.component();
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let _: Arc<String> = c.resolve().unwrap();
}

#[test]
fn resolver_resolve_qualified_undeclared() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|resolver| {
        let q = Qualifier::new("q").unwrap();
        let result: Result<Arc<i32>, _> = resolver.resolve_qualified(&q);
        assert!(result.is_err());
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let _: Arc<String> = c.resolve().unwrap();
}

#[test]
fn resolver_resolve_optional_qualified_undeclared() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|resolver| {
        let q = Qualifier::new("q").unwrap();
        let result: Result<Option<Arc<i32>>, _> = resolver.resolve_optional_qualified(&q);
        assert!(result.is_err());
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let _: Arc<String> = c.resolve().unwrap();
}

#[test]
fn resolver_resolve_trait_undeclared() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|resolver| {
        let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> = resolver.resolve_trait();
        assert!(result.is_err());
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let _: Arc<String> = c.resolve().unwrap();
}

#[test]
fn resolver_resolve_qualified_trait_undeclared() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|resolver| {
        let q = Qualifier::new("q").unwrap();
        let result: Result<Arc<dyn std::fmt::Display + Send + Sync>, _> =
            resolver.resolve_qualified_trait(&q);
        assert!(result.is_err());
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let _: Arc<String> = c.resolve().unwrap();
}

#[test]
fn resolver_resolve_optional_trait_undeclared() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|resolver| {
        let result: Result<Option<Arc<dyn std::fmt::Display + Send + Sync>>, _> =
            resolver.resolve_optional_trait();
        assert!(result.is_err());
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let _: Arc<String> = c.resolve().unwrap();
}

#[test]
fn resolver_resolve_optional_qualified_trait_undeclared() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|resolver| {
        let q = Qualifier::new("q").unwrap();
        let result: Result<Option<Arc<dyn std::fmt::Display + Send + Sync>>, _> =
            resolver.resolve_optional_qualified_trait(&q);
        assert!(result.is_err());
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let _: Arc<String> = c.resolve().unwrap();
}

#[test]
fn resolver_resolve_all_traits_undeclared() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|resolver| {
        let result: Result<Vec<Arc<dyn std::fmt::Display + Send + Sync>>, _> =
            resolver.resolve_all_traits();
        assert!(result.is_err());
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let _: Arc<String> = c.resolve().unwrap();
}

#[test]
fn resolver_provider_undeclared() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|resolver| {
        let result = resolver.provider::<i32>();
        assert!(result.is_err());
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let _: Arc<String> = c.resolve().unwrap();
}

#[test]
fn resolver_qualified_provider_undeclared() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|resolver| {
        let q = Qualifier::new("q").unwrap();
        let result = resolver.qualified_provider::<i32>(&q);
        assert!(result.is_err());
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let _: Arc<String> = c.resolve().unwrap();
}

#[test]
fn resolver_optional_provider_undeclared() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|resolver| {
        let result = resolver.optional_provider::<i32>();
        assert!(result.is_err());
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let _: Arc<String> = c.resolve().unwrap();
}

#[test]
fn resolver_optional_qualified_provider_undeclared() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|resolver| {
        let q = Qualifier::new("q").unwrap();
        let result = resolver.optional_qualified_provider::<i32>(&q);
        assert!(result.is_err());
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let _: Arc<String> = c.resolve().unwrap();
}

#[test]
fn resolver_trait_provider_undeclared() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|resolver| {
        let result = resolver.trait_provider::<dyn std::fmt::Display + Send + Sync>();
        assert!(result.is_err());
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let _: Arc<String> = c.resolve().unwrap();
}

#[test]
fn resolver_qualified_trait_provider_undeclared() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|resolver| {
        let q = Qualifier::new("q").unwrap();
        let result = resolver.qualified_trait_provider::<dyn std::fmt::Display + Send + Sync>(&q);
        assert!(result.is_err());
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let _: Arc<String> = c.resolve().unwrap();
}

#[test]
fn resolver_optional_trait_provider_undeclared() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|resolver| {
        let result = resolver.optional_trait_provider::<dyn std::fmt::Display + Send + Sync>();
        assert!(result.is_err());
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let _: Arc<String> = c.resolve().unwrap();
}

#[test]
fn resolver_optional_qualified_trait_provider_undeclared() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|resolver| {
        let q = Qualifier::new("q").unwrap();
        let result = resolver.optional_qualified_trait_provider::<dyn std::fmt::Display + Send + Sync>(&q);
        assert!(result.is_err());
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let _: Arc<String> = c.resolve().unwrap();
}

// ═══════════════════════════════════════════════════════════════════════════════
// 7. custom_number_editor.rs — Additional edge cases
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn number_editor_f64_parse_error() {
    use vernal_beans::propertyeditors::custom_number_editor::*;
    let mut editor = CustomNumberEditor::new(NumberType::F64);
    assert!(editor.set_as_text("not_a_number").is_err());
}

#[test]
fn number_editor_empty_not_allowed() {
    use vernal_beans::propertyeditors::custom_number_editor::*;
    let mut editor = CustomNumberEditor::new(NumberType::I64).with_allow_empty(false);
    assert!(editor.set_as_text("").is_err());
}

#[test]
fn number_editor_set_value_i64() {
    use vernal_beans::propertyeditors::custom_number_editor::*;
    let mut editor = CustomNumberEditor::new(NumberType::I64);
    let val: Arc<dyn Any + Send + Sync> = Arc::new(42i64);
    editor.set_value(val);
    assert_eq!(editor.as_i64(), Some(42));
}

#[test]
fn number_editor_set_value_f64() {
    use vernal_beans::propertyeditors::custom_number_editor::*;
    let mut editor = CustomNumberEditor::new(NumberType::F64);
    let val: Arc<dyn Any + Send + Sync> = Arc::new(3.14f64);
    editor.set_value(val);
    assert!((editor.as_f64().unwrap() - 3.14).abs() < f64::EPSILON);
}

#[test]
fn number_editor_set_value_wrong_type() {
    use vernal_beans::propertyeditors::custom_number_editor::*;
    let mut editor = CustomNumberEditor::new(NumberType::I64);
    let val: Arc<dyn Any + Send + Sync> = Arc::new(String::from("not a number"));
    editor.set_value(val);
    assert!(editor.as_i64().is_none());
}

#[test]
fn number_editor_get_value_none() {
    use vernal_beans::propertyeditors::custom_number_editor::*;
    let editor = CustomNumberEditor::new(NumberType::I64);
    assert!(editor.get_value().is_none());
}

#[test]
fn number_editor_get_value_some() {
    use vernal_beans::propertyeditors::custom_number_editor::*;
    let mut editor = CustomNumberEditor::new(NumberType::I64);
    editor.set_as_text("42").unwrap();
    assert!(editor.get_value().is_some());
}

#[test]
fn number_editor_get_value_type_i64() {
    use vernal_beans::propertyeditors::custom_number_editor::*;
    let editor = CustomNumberEditor::new(NumberType::I64);
    assert_eq!(editor.get_value_type(), TypeId::of::<i64>());
}

#[test]
fn number_editor_get_value_type_f64() {
    use vernal_beans::propertyeditors::custom_number_editor::*;
    let editor = CustomNumberEditor::new(NumberType::F64);
    assert_eq!(editor.get_value_type(), TypeId::of::<f64>());
}

#[test]
fn number_editor_get_as_text_f64() {
    use vernal_beans::propertyeditors::custom_number_editor::*;
    let mut editor = CustomNumberEditor::new(NumberType::F64);
    editor.set_as_text("3.14").unwrap();
    let text = editor.get_as_text().unwrap();
    assert!(text.contains("3.14"));
}

#[test]
fn number_editor_get_as_text_none() {
    use vernal_beans::propertyeditors::custom_number_editor::*;
    let editor = CustomNumberEditor::new(NumberType::I64);
    assert!(editor.get_as_text().is_none());
}

#[test]
fn number_editor_number_type() {
    use vernal_beans::propertyeditors::custom_number_editor::*;
    let editor = CustomNumberEditor::new(NumberType::F64);
    assert_eq!(editor.number_type(), NumberType::F64);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 8. factory/config/bean_definition.rs — BeanDefinition interface
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bean_definition_role_constants() {
    use vernal_beans::factory::config::bean_definition::*;
    assert_eq!(ROLE_APPLICATION, 0);
    assert_eq!(ROLE_SUPPORT, 1);
    assert_eq!(ROLE_INFRASTRUCTURE, 2);
}

#[test]
fn bean_definition_scope_constants() {
    use vernal_beans::factory::config::bean_definition::*;
    assert_eq!(SCOPE_SINGLETON, "singleton");
    assert_eq!(SCOPE_PROTOTYPE, "prototype");
}

#[test]
fn bean_definition_default_methods() {
    use vernal_beans::BeanDefinition;
    let rbd = vernal_beans::RootBeanDefinition::new();
    assert!(!rbd.is_fallback());
    assert!(rbd.is_autowire_candidate());
    assert_eq!(rbd.role(), 0);
    assert!(rbd.description().is_none());
    assert!(rbd.bean_class_name_internal().is_some());
    assert!(rbd.parent_name().is_none());
    assert!(rbd.factory_bean_name().is_none());
    assert!(rbd.factory_method_name().is_none());
    assert!(rbd.init_method_name().is_none());
    assert!(rbd.destroy_method_name().is_none());
    assert!(rbd.resource_description().is_none());
    assert!(rbd.originating_bean_definition().is_none());
}

#[test]
fn bean_definition_is_singleton_is_prototype() {
    use vernal_beans::BeanDefinition;
    let mut rbd = vernal_beans::RootBeanDefinition::new();
    assert!(rbd.is_singleton());
    assert!(!rbd.is_prototype());
    rbd.set_scope(Scope::Transient);
    assert!(!rbd.is_singleton());
    assert!(rbd.is_prototype());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 9. support/paged_list_holder.rs — Paged list
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn paged_list_holder_new_default_page_size() {
    use vernal_beans::support::paged_list_holder::PagedListHolder;
    let data: Vec<i32> = (0..5).collect();
    let holder = PagedListHolder::new(data);
    assert_eq!(holder.page_size(), 5);
    assert_eq!(holder.page_count(), 1);
}

#[test]
fn paged_list_holder_previous_page() {
    use vernal_beans::support::paged_list_holder::PagedListHolder;
    let data: Vec<i32> = (0..10).collect();
    let mut holder = PagedListHolder::with_page_size(data, 3);
    holder.next_page();
    holder.next_page();
    assert_eq!(holder.current_page(), 2);
    holder.previous_page();
    assert_eq!(holder.current_page(), 1);
}

#[test]
fn paged_list_holder_previous_page_at_start() {
    use vernal_beans::support::paged_list_holder::PagedListHolder;
    let data: Vec<i32> = (0..10).collect();
    let mut holder = PagedListHolder::with_page_size(data, 3);
    holder.previous_page();
    assert_eq!(holder.current_page(), 0);
}

#[test]
fn paged_list_holder_next_page_at_end() {
    use vernal_beans::support::paged_list_holder::PagedListHolder;
    let data: Vec<i32> = (0..3).collect();
    let mut holder = PagedListHolder::with_page_size(data, 3);
    holder.next_page();
    assert_eq!(holder.current_page(), 0);
}

#[test]
fn paged_list_holder_set_current_page() {
    use vernal_beans::support::paged_list_holder::PagedListHolder;
    let data: Vec<i32> = (0..10).collect();
    let mut holder = PagedListHolder::with_page_size(data, 3);
    holder.set_current_page(3);
    assert_eq!(holder.current_page(), 3);
}

#[test]
fn paged_list_holder_set_current_page_beyond_max() {
    use vernal_beans::support::paged_list_holder::PagedListHolder;
    let data: Vec<i32> = (0..10).collect();
    let mut holder = PagedListHolder::with_page_size(data, 3);
    holder.set_current_page(100);
    assert_eq!(holder.current_page(), 3);
}

#[test]
fn paged_list_holder_source() {
    use vernal_beans::support::paged_list_holder::PagedListHolder;
    let data: Vec<i32> = (0..5).collect();
    let holder = PagedListHolder::new(data.clone());
    assert_eq!(holder.source(), &data[..]);
}

#[test]
fn paged_list_holder_source_size() {
    use vernal_beans::support::paged_list_holder::PagedListHolder;
    let data: Vec<i32> = (0..5).collect();
    let holder = PagedListHolder::new(data);
    assert_eq!(holder.source_size(), 5);
}

#[test]
fn paged_list_holder_sort_property() {
    use vernal_beans::support::paged_list_holder::PagedListHolder;
    let data: Vec<i32> = (0..5).collect();
    let mut holder = PagedListHolder::new(data);
    holder.set_sort_property("name");
    holder.set_ascending(false);
    assert!(!holder.is_ascending());
}

#[test]
fn paged_list_holder_set_page_size_clamps_current_page() {
    use vernal_beans::support::paged_list_holder::PagedListHolder;
    let data: Vec<i32> = (0..10).collect();
    let mut holder = PagedListHolder::with_page_size(data, 2);
    holder.set_current_page(4);
    holder.set_page_size(5);
    assert_eq!(holder.current_page(), 1);
}

#[test]
fn paged_list_holder_empty_page_navigation() {
    use vernal_beans::support::paged_list_holder::PagedListHolder;
    let holder: PagedListHolder<i32> = PagedListHolder::new(vec![]);
    assert!(!holder.has_next_page());
    assert!(!holder.has_previous_page());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 10. custom_date_editor.rs — Additional edge cases
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn date_editor_default() {
    let editor = vernal_beans::propertyeditors::custom_date_editor::CustomDateEditor::new();
    assert!(editor.get_as_text().is_none());
    assert!(editor.get_value().is_none());
}

#[test]
fn date_editor_empty_not_allowed() {
    let mut editor = vernal_beans::propertyeditors::custom_date_editor::CustomDateEditor::new()
        .with_allow_empty(false);
    assert!(editor.set_as_text("").is_err());
}

#[test]
fn date_editor_set_value() {
    let mut editor = vernal_beans::propertyeditors::custom_date_editor::CustomDateEditor::new();
    let val: Arc<dyn Any + Send + Sync> = Arc::new(1704067200000i64);
    editor.set_value(val);
    assert!(editor.get_value().is_some());
}

#[test]
fn date_editor_set_value_wrong_type() {
    let mut editor = vernal_beans::propertyeditors::custom_date_editor::CustomDateEditor::new();
    let val: Arc<dyn Any + Send + Sync> = Arc::new(String::from("not a timestamp"));
    editor.set_value(val);
    assert!(editor.get_value().is_none());
}

#[test]
fn date_editor_target_type() {
    let editor = vernal_beans::propertyeditors::custom_date_editor::CustomDateEditor::new();
    assert_eq!(editor.target_type(), TypeId::of::<i64>());
}

#[test]
fn date_editor_get_value_type() {
    let editor = vernal_beans::propertyeditors::custom_date_editor::CustomDateEditor::new();
    assert_eq!(editor.get_value_type(), TypeId::of::<i64>());
}

#[test]
fn date_editor_get_as_text_some() {
    let mut editor = vernal_beans::propertyeditors::custom_date_editor::CustomDateEditor::new();
    editor.set_as_text("12345").unwrap();
    assert_eq!(editor.get_as_text(), Some("12345".to_string()));
}

#[test]
fn date_editor_invalid_iso_date() {
    let mut editor = vernal_beans::propertyeditors::custom_date_editor::CustomDateEditor::new();
    assert!(editor.set_as_text("2024-13-01").is_err());
}

#[test]
fn date_editor_invalid_iso_date_day() {
    let mut editor = vernal_beans::propertyeditors::custom_date_editor::CustomDateEditor::new();
    assert!(editor.set_as_text("2024-01-32").is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 11. destructible_bean_adapter.rs
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn destructible_adapter_with_callback() {
    use vernal_beans::destructible_bean_adapter::DisposableBeanAdapter;
    let callback: Arc<dyn Fn() -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync> =
        Arc::new(|| Ok(()));
    let adapter = DisposableBeanAdapter::with_callback(
        "myBean",
        Arc::new(42i32),
        Some("destroy".to_string()),
        callback,
    );
    assert_eq!(adapter.bean_name(), "myBean");
    assert_eq!(adapter.destroy_method_name(), Some("destroy"));
    assert!(adapter.destroy().is_ok());
}

#[test]
fn destructible_adapter_callback_error() {
    use vernal_beans::destructible_bean_adapter::DisposableBeanAdapter;
    let callback: Arc<dyn Fn() -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync> =
        Arc::new(|| Err("destroy failed".into()));
    let adapter = DisposableBeanAdapter::with_callback(
        "myBean",
        Arc::new(42i32),
        Some("destroy".to_string()),
        callback,
    );
    assert!(adapter.destroy().is_err());
}

#[test]
fn destructible_adapter_debug_format() {
    use vernal_beans::destructible_bean_adapter::DisposableBeanAdapter;
    let adapter = DisposableBeanAdapter::new("myBean", Arc::new(42i32), Some("destroy".to_string()));
    let debug = format!("{:?}", adapter);
    assert!(debug.contains("DisposableBeanAdapter"));
    assert!(debug.contains("myBean"));
}

#[test]
fn destructible_adapter_no_destroy_method() {
    use vernal_beans::destructible_bean_adapter::DisposableBeanAdapter;
    let adapter = DisposableBeanAdapter::new("myBean", Arc::new(42i32), None);
    assert!(adapter.destroy_method_name().is_none());
}

#[test]
fn destructible_adapter_no_callback() {
    use vernal_beans::destructible_bean_adapter::DisposableBeanAdapter;
    let adapter = DisposableBeanAdapter::new("myBean", Arc::new(42i32), None);
    assert!(adapter.destroy().is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 12. configuration_class_post_processor.rs
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn configuration_class_post_processor_default() {
    use vernal_beans::configuration_class_post_processor::ConfigurationClassPostProcessor;
    let p = ConfigurationClassPostProcessor::default();
    assert_eq!(p.registered_count(), 0);
    assert!(p.registered_configurations().is_empty());
}

#[test]
fn configuration_class_post_processor_register() {
    use vernal_beans::configuration_class_post_processor::ConfigurationClassPostProcessor;
    let mut p = ConfigurationClassPostProcessor::new();
    p.register_configuration("com.example.AppConfig");
    assert_eq!(p.registered_count(), 1);
    assert_eq!(p.registered_configurations()[0], "com.example.AppConfig");
}

#[test]
fn configuration_class_post_processor_multiple() {
    use vernal_beans::configuration_class_post_processor::ConfigurationClassPostProcessor;
    let mut p = ConfigurationClassPostProcessor::new();
    p.register_configuration("config1");
    p.register_configuration("config2");
    assert_eq!(p.registered_count(), 2);
}

#[test]
fn configuration_class_post_processor_bean_factory_post_processor() {
    use vernal_beans::configuration_class_post_processor::ConfigurationClassPostProcessor;
    use vernal_beans::BeanFactoryPostProcessor;
    let p = ConfigurationClassPostProcessor::new();
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    let mut c = Container::new(b.build().unwrap());
    let result = p.post_process_bean_factory(&mut c);
    assert!(result.is_ok());
}

#[test]
fn configuration_class_post_processor_registry_post_processor() {
    use vernal_beans::configuration_class_post_processor::ConfigurationClassPostProcessor;
    use vernal_beans::factory::support::bean_definition_registry_post_processor::BeanDefinitionRegistryPostProcessor;
    let p = ConfigurationClassPostProcessor::new();
    let mut b = RegistryBuilder::new();
    let result = p.post_process_bean_definition_registry(&mut b);
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 13. path_property_editor.rs
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn path_editor_default() {
    let editor = vernal_beans::path_property_editor::PathPropertyEditor::default();
    assert!(editor.get_as_text().is_none());
    assert!(editor.get_value().is_none());
}

#[test]
fn path_editor_set_and_get() {
    let mut editor = vernal_beans::path_property_editor::PathPropertyEditor::new();
    editor.set_as_text("/usr/local/bin").unwrap();
    assert_eq!(editor.get_as_text(), Some("/usr/local/bin".to_string()));
}

#[test]
fn path_editor_empty_string() {
    let mut editor = vernal_beans::path_property_editor::PathPropertyEditor::new();
    editor.set_as_text("").unwrap();
    assert_eq!(editor.get_as_text(), Some("".to_string()));
}

#[test]
fn path_editor_set_value() {
    let mut editor = vernal_beans::path_property_editor::PathPropertyEditor::new();
    let val: Arc<dyn Any + Send + Sync> = Arc::new(String::from("/tmp"));
    editor.set_value(val);
    assert_eq!(editor.get_as_text(), Some("/tmp".to_string()));
}

#[test]
fn path_editor_set_value_wrong_type() {
    let mut editor = vernal_beans::path_property_editor::PathPropertyEditor::new();
    let val: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    editor.set_value(val);
    assert!(editor.get_as_text().is_none());
}

#[test]
fn path_editor_target_type() {
    let editor = vernal_beans::path_property_editor::PathPropertyEditor::new();
    assert_eq!(editor.target_type(), TypeId::of::<String>());
}

#[test]
fn path_editor_get_value_type() {
    let editor = vernal_beans::path_property_editor::PathPropertyEditor::new();
    assert_eq!(editor.get_value_type(), TypeId::of::<String>());
}

#[test]
fn path_editor_get_value_some() {
    let mut editor = vernal_beans::path_property_editor::PathPropertyEditor::new();
    editor.set_as_text("/path").unwrap();
    assert!(editor.get_value().is_some());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 14. file_array_editor.rs
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn file_array_editor_default() {
    let editor = vernal_beans::file_array_editor::FileArrayEditor::default();
    assert!(editor.get_as_text().is_none());
    assert!(editor.get_value().is_none());
}

#[test]
fn file_array_editor_set_and_get() {
    let mut editor = vernal_beans::file_array_editor::FileArrayEditor::new();
    editor.set_as_text("file1.txt,file2.txt").unwrap();
    assert_eq!(
        editor.get_as_text(),
        Some("file1.txt,file2.txt".to_string())
    );
}

#[test]
fn file_array_editor_empty_string() {
    let mut editor = vernal_beans::file_array_editor::FileArrayEditor::new();
    editor.set_as_text("").unwrap();
    assert_eq!(editor.get_as_text(), Some("".to_string()));
}

#[test]
fn file_array_editor_set_value() {
    let mut editor = vernal_beans::file_array_editor::FileArrayEditor::new();
    let val: Arc<dyn Any + Send + Sync> = Arc::new(String::from("/tmp/file.txt"));
    editor.set_value(val);
    assert_eq!(editor.get_as_text(), Some("/tmp/file.txt".to_string()));
}

#[test]
fn file_array_editor_set_value_wrong_type() {
    let mut editor = vernal_beans::file_array_editor::FileArrayEditor::new();
    let val: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    editor.set_value(val);
    assert!(editor.get_as_text().is_none());
}

#[test]
fn file_array_editor_target_type() {
    let editor = vernal_beans::file_array_editor::FileArrayEditor::new();
    assert_eq!(editor.target_type(), TypeId::of::<String>());
}

#[test]
fn file_array_editor_get_value_type() {
    let editor = vernal_beans::file_array_editor::FileArrayEditor::new();
    assert_eq!(editor.get_value_type(), TypeId::of::<String>());
}

#[test]
fn file_array_editor_get_value_some() {
    let mut editor = vernal_beans::file_array_editor::FileArrayEditor::new();
    editor.set_as_text("file.txt").unwrap();
    assert!(editor.get_value().is_some());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 15. factory/config/property_placeholder_configurer.rs
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn placeholder_configurer_default() {
    use vernal_beans::factory::config::property_placeholder_configurer::PropertyPlaceholderConfigurer;
    let c = PropertyPlaceholderConfigurer::default();
    assert!(c.get_property("key").is_none());
}

#[test]
fn placeholder_configurer_set_properties() {
    use vernal_beans::factory::config::property_placeholder_configurer::PropertyPlaceholderConfigurer;
    let mut c = PropertyPlaceholderConfigurer::new();
    let mut props = HashMap::new();
    props.insert("key".to_string(), "value".to_string());
    c.set_properties(props);
    assert_eq!(c.get_property("key"), Some("value"));
}

#[test]
fn placeholder_configurer_multiple_placeholders() {
    use vernal_beans::factory::config::property_placeholder_configurer::PropertyPlaceholderConfigurer;
    let mut c = PropertyPlaceholderConfigurer::new();
    c.set_property("host", "localhost");
    c.set_property("port", "8080");
    let result = c.resolve_placeholders("http://${host}:${port}");
    assert_eq!(result, "http://localhost:8080");
}

#[test]
fn placeholder_configurer_ignore_unresolvable() {
    use vernal_beans::factory::config::property_placeholder_configurer::PropertyPlaceholderConfigurer;
    let mut c = PropertyPlaceholderConfigurer::new();
    c.set_ignore_unresolvable(true);
    let result = c.resolve_placeholders("${unknown}");
    assert_eq!(result, "${unknown}");
}

#[test]
fn placeholder_configurer_not_ignore_unresolvable() {
    use vernal_beans::factory::config::property_placeholder_configurer::PropertyPlaceholderConfigurer;
    let c = PropertyPlaceholderConfigurer::new();
    let result = c.resolve_placeholders("${unknown}");
    assert_eq!(result, "${unknown}");
}

#[test]
fn placeholder_configurer_no_closing_brace() {
    use vernal_beans::factory::config::property_placeholder_configurer::PropertyPlaceholderConfigurer;
    let c = PropertyPlaceholderConfigurer::new();
    let result = c.resolve_placeholders("${unclosed");
    assert_eq!(result, "${unclosed");
}

#[test]
fn placeholder_configurer_custom_prefix_suffix() {
    use vernal_beans::factory::config::property_placeholder_configurer::PropertyPlaceholderConfigurer;
    let mut c = PropertyPlaceholderConfigurer::new();
    c.set_placeholder_prefix("{{");
    c.set_placeholder_suffix("}}");
    c.set_property("key", "value");
    let result = c.resolve_placeholders("{{key}}");
    assert_eq!(result, "value");
}

#[test]
fn placeholder_configurer_custom_separator() {
    use vernal_beans::factory::config::property_placeholder_configurer::PropertyPlaceholderConfigurer;
    let mut c = PropertyPlaceholderConfigurer::new();
    c.set_value_separator("|");
    let result = c.resolve_placeholders("${missing|fallback}");
    assert_eq!(result, "fallback");
}

#[test]
fn placeholder_configurer_system_properties_mode() {
    use vernal_beans::factory::config::property_placeholder_configurer::*;
    assert_eq!(SystemPropertiesMode::default(), SystemPropertiesMode::Fallback);
}

#[test]
fn placeholder_configurer_set_system_properties_mode() {
    use vernal_beans::factory::config::property_placeholder_configurer::*;
    let mut c = PropertyPlaceholderConfigurer::new();
    c.set_system_properties_mode(SystemPropertiesMode::Override);
    c.set_system_properties_mode(SystemPropertiesMode::Never);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 16. bean_info_factory.rs
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bean_info_entries_default() {
    use vernal_beans::bean_info_factory::BeanInfoEntries;
    let entries = BeanInfoEntries::default();
    assert_eq!(entries.property_count(), 0);
    assert_eq!(entries.method_count(), 0);
}

#[test]
fn bean_info_entries_add_multiple() {
    use vernal_beans::bean_info_factory::*;
    let mut entries = BeanInfoEntries::new();
    entries.add_property(PropertyDescriptorEntry::new("name", TypeId::of::<String>(), true, true));
    entries.add_property(PropertyDescriptorEntry::new("age", TypeId::of::<i32>(), true, false));
    entries.add_method(MethodDescriptorEntry::new("getName", 0));
    entries.add_method(MethodDescriptorEntry::new("setAge", 1));
    assert_eq!(entries.property_count(), 2);
    assert_eq!(entries.method_count(), 2);
}

#[test]
fn property_descriptor_entry_methods() {
    use vernal_beans::bean_info_factory::PropertyDescriptorEntry;
    let entry = PropertyDescriptorEntry::new("name", TypeId::of::<String>(), true, false);
    assert_eq!(entry.name(), "name");
    assert_eq!(entry.property_type(), TypeId::of::<String>());
    assert!(entry.is_readable());
    assert!(!entry.is_writable());
}

#[test]
fn method_descriptor_entry_methods() {
    use vernal_beans::bean_info_factory::MethodDescriptorEntry;
    let entry = MethodDescriptorEntry::new("doSomething", 3);
    assert_eq!(entry.name(), "doSomething");
    assert_eq!(entry.parameter_count(), 3);
}

#[test]
fn simple_bean_info_factory_returns_empty() {
    use vernal_beans::bean_info_factory::*;
    let factory = SimpleBeanInfoFactory;
    let entries = factory.get_bean_info_entries(TypeId::of::<i32>()).unwrap();
    assert_eq!(entries.property_count(), 0);
    assert_eq!(entries.method_count(), 0);
}

#[test]
fn bean_info_entries_properties_and_methods() {
    use vernal_beans::bean_info_factory::*;
    let mut entries = BeanInfoEntries::new();
    entries.add_property(PropertyDescriptorEntry::new("x", TypeId::of::<i32>(), true, true));
    entries.add_method(MethodDescriptorEntry::new("calc", 2));
    assert_eq!(entries.properties().len(), 1);
    assert_eq!(entries.methods().len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 17. type_editor_registry.rs — Additional edge cases
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn type_editor_registry_find_nonexistent() {
    let registry = vernal_beans::TypeEditorRegistry::new();
    assert!(registry.find_editor_by_type(TypeId::of::<i32>()).is_none());
    assert!(registry.find_editor_by_name("nonexistent").is_none());
}

#[test]
fn type_editor_registry_resolve_alias_nonexistent() {
    let registry = vernal_beans::TypeEditorRegistry::new();
    assert!(registry.resolve_alias("nonexistent").is_none());
}

#[test]
fn type_editor_registry_has_editor_for_name_alias() {
    let registry = vernal_beans::TypeEditorRegistry::new();
    registry.register_alias("int", "i32");
    assert!(!registry.has_editor_for_name("int"));
}

#[test]
fn type_editor_registry_has_editor_for_type_false() {
    let registry = vernal_beans::TypeEditorRegistry::new();
    assert!(!registry.has_editor_for_type(TypeId::of::<i32>()));
}

#[test]
fn type_editor_registry_debug_format() {
    let registry = vernal_beans::TypeEditorRegistry::new();
    let debug = format!("{:?}", registry);
    assert!(debug.contains("TypeEditorRegistry"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 18. property_editor_registry.rs — Additional edge cases
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn property_editor_registry_find_not_found() {
    use vernal_beans::property_editor_registry::*;
    let registry = SimplePropertyEditorRegistry::new();
    assert!(!registry.has_custom_editor(TypeId::of::<String>(), None));
    assert!(registry.find_custom_editor(TypeId::of::<String>(), None).is_none());
}

#[test]
fn property_editor_registry_find_mut_not_found() {
    use vernal_beans::property_editor_registry::*;
    let mut registry = SimplePropertyEditorRegistry::new();
    assert!(registry.find_custom_editor_mut(TypeId::of::<String>(), None).is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 19. factory/xml/pluggable_schema_resolver.rs
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn pluggable_schema_resolver_new() {
    use vernal_beans::factory::xml::pluggable_schema_resolver::PluggableSchemaResolver;
    let resolver = PluggableSchemaResolver::new();
    assert!(resolver.schema_mappings.is_empty());
}

#[test]
fn pluggable_schema_resolver_from_text() {
    use vernal_beans::factory::xml::pluggable_schema_resolver::PluggableSchemaResolver;
    let text = "http://example.com/schema.xsd=/path/to/schema.xsd\nhttp://example.com/other.xsd=/other/path.xsd";
    let resolver = PluggableSchemaResolver::from_text(text);
    assert_eq!(resolver.schema_mappings.len(), 2);
    assert_eq!(
        resolver.schema_mappings.get("http://example.com/schema.xsd"),
        Some(&"/path/to/schema.xsd".to_string())
    );
}

#[test]
fn pluggable_schema_resolver_from_text_comments() {
    use vernal_beans::factory::xml::pluggable_schema_resolver::PluggableSchemaResolver;
    let text = "# This is a comment\nhttp://example.com/schema.xsd=/path/to/schema.xsd\n# Another comment";
    let resolver = PluggableSchemaResolver::from_text(text);
    assert_eq!(resolver.schema_mappings.len(), 1);
}

#[test]
fn pluggable_schema_resolver_from_text_empty_lines() {
    use vernal_beans::factory::xml::pluggable_schema_resolver::PluggableSchemaResolver;
    let text = "\n\nhttp://example.com/schema.xsd=/path\n\n";
    let resolver = PluggableSchemaResolver::from_text(text);
    assert_eq!(resolver.schema_mappings.len(), 1);
}

#[test]
fn pluggable_schema_resolver_from_text_whitespace() {
    use vernal_beans::factory::xml::pluggable_schema_resolver::PluggableSchemaResolver;
    let text = "  http://example.com/schema.xsd = /path/to/schema.xsd  ";
    let resolver = PluggableSchemaResolver::from_text(text);
    assert_eq!(resolver.schema_mappings.len(), 1);
}

#[test]
fn pluggable_schema_resolver_resolve_entity_found() {
    use vernal_beans::factory::xml::pluggable_schema_resolver::PluggableSchemaResolver;
    use vernal_beans::factory::xml::entity_resolver::EntityResolver;
    let resolver = PluggableSchemaResolver::from_text(
        "http://example.com/schema.xsd=/path/to/schema.xsd",
    );
    let result = resolver
        .resolve_entity(None, "http://example.com/schema.xsd")
        .unwrap();
    assert_eq!(result.content, b"/path/to/schema.xsd");
}

#[test]
fn pluggable_schema_resolver_resolve_entity_not_found() {
    use vernal_beans::factory::xml::pluggable_schema_resolver::PluggableSchemaResolver;
    use vernal_beans::factory::xml::entity_resolver::EntityResolver;
    let resolver = PluggableSchemaResolver::new();
    let result = resolver.resolve_entity(None, "http://unknown.xsd");
    assert!(result.is_err());
}

#[test]
fn pluggable_schema_resolver_default() {
    use vernal_beans::factory::xml::pluggable_schema_resolver::PluggableSchemaResolver;
    let resolver = PluggableSchemaResolver::default();
    assert!(resolver.schema_mappings.is_empty());
}

#[test]
fn pluggable_schema_resolver_clone() {
    use vernal_beans::factory::xml::pluggable_schema_resolver::PluggableSchemaResolver;
    let resolver = PluggableSchemaResolver::from_text("key=value");
    let cloned = resolver.clone();
    assert_eq!(cloned.schema_mappings.len(), 1);
}

#[test]
fn pluggable_schema_resolver_debug() {
    use vernal_beans::factory::xml::pluggable_schema_resolver::PluggableSchemaResolver;
    let resolver = PluggableSchemaResolver::new();
    let debug = format!("{:?}", resolver);
    assert!(debug.contains("PluggableSchemaResolver"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 20. custom_map_editor.rs — Additional edge cases
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn map_editor_default() {
    use vernal_beans::propertyeditors::custom_map_editor::CustomMapEditor;
    let editor = CustomMapEditor::default();
    assert!(editor.is_empty());
    assert_eq!(editor.len(), 0);
    assert_eq!(editor.get_as_text(), Some("{}".to_string()));
}

#[test]
fn map_editor_empty_not_allowed() {
    use vernal_beans::propertyeditors::custom_map_editor::CustomMapEditor;
    let mut editor = CustomMapEditor::new().with_allow_empty(false);
    assert!(editor.set_as_text("").is_err());
}

#[test]
fn map_editor_set_value() {
    use vernal_beans::propertyeditors::custom_map_editor::CustomMapEditor;
    let mut editor = CustomMapEditor::new();
    let mut map = HashMap::new();
    map.insert("key".to_string(), "value".to_string());
    let val: Arc<dyn Any + Send + Sync> = Arc::new(map);
    editor.set_value(val);
    assert_eq!(editor.len(), 1);
}

#[test]
fn map_editor_set_value_wrong_type() {
    use vernal_beans::propertyeditors::custom_map_editor::CustomMapEditor;
    let mut editor = CustomMapEditor::new();
    let val: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    editor.set_value(val);
    assert!(editor.is_empty());
}

#[test]
fn map_editor_get_value() {
    use vernal_beans::propertyeditors::custom_map_editor::CustomMapEditor;
    let editor = CustomMapEditor::new();
    assert!(editor.get_value().is_some());
}

#[test]
fn map_editor_target_type() {
    use vernal_beans::propertyeditors::custom_map_editor::CustomMapEditor;
    let editor = CustomMapEditor::new();
    assert_eq!(editor.target_type(), TypeId::of::<HashMap<String, String>>());
}

#[test]
fn map_editor_get_value_type() {
    use vernal_beans::propertyeditors::custom_map_editor::CustomMapEditor;
    let editor = CustomMapEditor::new();
    assert_eq!(editor.get_value_type(), TypeId::of::<HashMap<String, String>>());
}

#[test]
fn map_editor_json_parse_error() {
    use vernal_beans::propertyeditors::custom_map_editor::CustomMapEditor;
    let mut editor = CustomMapEditor::new();
    let result = editor.set_as_text("{invalid json");
    assert!(result.is_ok());
}

#[test]
fn map_editor_whitespace_trimming() {
    use vernal_beans::propertyeditors::custom_map_editor::CustomMapEditor;
    let mut editor = CustomMapEditor::new();
    editor.set_as_text("  key=val  ").unwrap();
    assert_eq!(editor.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Additional container.rs tests for deeper coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_object_provider_get_empty() {
    use vernal_beans::BeanFactory;
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let provider = c
        .get_bean_provider_by_type_id(TypeId::of::<String>())
        .unwrap();
    assert!(provider.get().is_err());
}

#[test]
fn container_object_provider_if_available_empty() {
    use vernal_beans::BeanFactory;
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let provider = c
        .get_bean_provider_by_type_id(TypeId::of::<String>())
        .unwrap();
    assert!(provider.if_available().is_none());
}

#[test]
fn container_object_provider_stream_empty() {
    use vernal_beans::BeanFactory;
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let provider = c
        .get_bean_provider_by_type_id(TypeId::of::<String>())
        .unwrap();
    assert!(provider.stream().is_empty());
}

#[test]
fn container_contains_local_bean_dynamic() {
    use vernal_beans::factory::hierarchical_bean_factory::HierarchicalBeanFactory;
    use vernal_beans::BeanDefinitionRegistry;
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    let def = Box::new(vernal_beans::RootBeanDefinition::new());
    c.register_bean_definition("dynamicBean".to_string(), def).unwrap();
    assert!(c.contains_local_bean("dynamicBean"));
}

#[test]
fn container_resolve_dependency_not_required() {
    use vernal_beans::factory::config::autowire_capable_bean_factory::AutowireCapableBeanFactory;
    use vernal_beans::factory::support::dependency_descriptor::DependencyDescriptor;
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let descriptor = DependencyDescriptor::new(
        TypeId::of::<String>(),
        "String".to_string(),
        false,
    );
    let result = c.resolve_dependency(&descriptor, None).unwrap();
    assert!(result.is_none());
}

#[test]
fn container_listable_beans_of_type_id_not_found() {
    use vernal_beans::ListableBeanFactory;
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let beans = c.beans_of_type_id(TypeId::of::<String>(), true, true).unwrap();
    assert!(beans.is_empty());
}

#[test]
fn container_listable_bean_names_for_type_id_empty() {
    use vernal_beans::ListableBeanFactory;
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let names = c.bean_names_for_type_id(TypeId::of::<String>(), true, true);
    assert!(names.is_empty());
}

#[test]
fn container_contains_non_singleton_bean_with_transient() {
    use vernal_beans::ListableBeanFactory;
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::transient::<String, _>(|_| "t".to_string()));
    let c = Container::new(b.build().unwrap());
    assert!(c.contains_non_singleton_bean());
}

#[test]
fn container_bean_definition_names_empty() {
    use vernal_beans::ListableBeanFactory;
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let names = c.bean_definition_names();
    assert!(names.is_empty());
}

#[test]
fn container_embedded_value_multiple_resolvers() {
    use vernal_beans::ConfigurableBeanFactory;
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    let r1: Arc<dyn Fn(&str) -> String + Send + Sync> = Arc::new(|v: &str| {
        v.replace("${a}", "A")
    });
    let r2: Arc<dyn Fn(&str) -> String + Send + Sync> = Arc::new(|v: &str| {
        v.replace("${b}", "B")
    });
    c.add_embedded_value_resolver(r1);
    c.add_embedded_value_resolver(r2);
    let result = c.resolve_embedded_value("${a}-${b}");
    assert_eq!(result, "A-B");
}

#[test]
fn container_listable_contains_bean_definition_dynamic() {
    use vernal_beans::ListableBeanFactory;
    use vernal_beans::BeanDefinitionRegistry;
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    let def = Box::new(vernal_beans::RootBeanDefinition::new());
    <Container as BeanDefinitionRegistry>::register_bean_definition(&mut c, "dynamicBean".to_string(), def).unwrap();
    assert!(<Container as ListableBeanFactory>::contains_bean_definition(&c, "dynamicBean"));
}

#[test]
fn container_listable_bean_definition_count_dynamic() {
    use vernal_beans::ListableBeanFactory;
    use vernal_beans::BeanDefinitionRegistry;
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    assert_eq!(<Container as ListableBeanFactory>::bean_definition_count(&c), 0);
    let def = Box::new(vernal_beans::RootBeanDefinition::new());
    <Container as BeanDefinitionRegistry>::register_bean_definition(&mut c, "dynamicBean".to_string(), def).unwrap();
    assert_eq!(<Container as ListableBeanFactory>::bean_definition_count(&c), 1);
}
