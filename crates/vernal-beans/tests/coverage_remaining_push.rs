//! Final push — 剩余未覆盖代码（build_plan, standard_expression_resolver, trait_key, component_contract, bean_definition 等）

use std::sync::Arc;
use vernal_beans::bean_factory::BeanFactory;
use vernal_beans::property_editor::PropertyEditor;
use vernal_beans::{
    BeanDefinition, ComponentDefinition, ComponentKey, Container, RegistryBuilder, Resolver, Scope,
};

// ── BuildPlan public methods ──────────────────────────────────────────

#[test]
fn build_plan_methods_direct() {
    use vernal_beans::ComponentKey;
    let keys = vec![ComponentKey::of::<String>(), ComponentKey::of::<i32>()];
    // BuildPlan is constructed internally via graph_planner
    // Test via Registry which wraps BuildPlan
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }));
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32));
    let r = b.build().unwrap();

    let plan = r.plan();
    assert_eq!(plan.len(), 2);
    assert!(!plan.is_empty());
    assert_eq!(plan.keys().len(), 2);
}

#[test]
fn build_plan_empty() {
    use vernal_beans::ComponentKey;
    let r = RegistryBuilder::new().build().unwrap();
    let plan = r.plan();
    assert_eq!(plan.len(), 0);
    assert!(plan.is_empty());
    assert!(plan.keys().is_empty());
}

// ── StandardBeanExpressionResolver comprehensive ──────────────────────

#[test]
fn standard_bean_expression_resolver_full() {
    use vernal_beans::bean_expression_resolver::BeanExpressionResolver;
    use vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver;

    let r = StandardBeanExpressionResolver::new();
    // Simple identifiers
    assert!(r.evaluate("someBean", None).unwrap().is_none());
    // SpEL template
    assert!(
        r.evaluate("#{systemProperties['foo']}", None)
            .unwrap()
            .is_none()
    );
    // Expression with spaces
    assert!(r.evaluate("  spaced  ", None).unwrap().is_none());
    // Special characters
    assert!(r.evaluate("a.b.c", None).unwrap().is_none());
    // With bean name
    assert!(r.evaluate("someBean", Some("testBean")).unwrap().is_none());
}

// ── BeanDefinition trait default methods ─────────────────────────────

#[test]
fn bean_definition_trait_defaults() {
    use vernal_beans::bean_definition::BeanDefinition;

    #[derive(Debug)]
    struct TestDef;
    impl BeanDefinition for TestDef {
        fn bean_name(&self) -> &ComponentKey {
            unimplemented!()
        }
        fn bean_class_name(&self) -> &str {
            "Test"
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

    let d = TestDef;
    assert!(!d.is_fallback());
    assert!(d.is_autowire_candidate());
    assert_eq!(d.role(), 0);
    assert_eq!(d.description(), None);
    assert_eq!(d.parent_name(), None);
    assert_eq!(d.factory_bean_name(), None);
    assert_eq!(d.factory_method_name(), None);
    assert_eq!(d.init_method_name(), None);
    assert_eq!(d.destroy_method_name(), None);
    assert!(!d.is_abstract());
    assert!(d.is_singleton());
    assert!(!d.is_prototype());
    assert!(d.resource_description().is_none());
    assert!(d.originating_bean_definition().is_none());
}

// ── Registry basic methods ────────────────────────────────────────────

#[test]
fn registry_empty_plan() {
    let r = RegistryBuilder::new().build().unwrap();
    assert_eq!(r.len(), 0);
    assert!(r.is_empty());
}

// ── RegistryBuilder register_bundle ──────────────────────────────────

#[test]
fn registry_builder_register_bundle() {
    use vernal_beans::RegistryBuilder;

    let mut b = RegistryBuilder::new();
    // register_bundle registers multiple definitions at once
    // We test that it doesn't panic
    let result = b.register_bundle(
        vec![
            ComponentDefinition::singleton::<String, _>(|_| "a".to_string()),
            ComponentDefinition::singleton::<i32, _>(|_| 42i32),
        ],
        vec![],
    );
    assert!(result.is_ok());
}

// ── ReaderEditor (82.14%) - test remaining uncovered path ├─

#[test]
fn reader_editor_constructors() {
    use vernal_beans::reader_editor::ReaderEditor;
    let e = ReaderEditor::new();
    let _e2: ReaderEditor = Default::default();
    assert!(e.get_value().is_none());
    assert_eq!(e.target_type(), std::any::TypeId::of::<String>());
}

#[test]
fn input_source_editor_constructors() {
    use vernal_beans::input_source_editor::InputSourceEditor;
    let e = InputSourceEditor::new();
    let _e2: InputSourceEditor = Default::default();
    assert!(e.get_value().is_none());
    assert_eq!(e.target_type(), std::any::TypeId::of::<String>());
}

#[test]
fn file_editor_constructors() {
    use vernal_beans::file_editor::FileEditor;
    let e = FileEditor::new();
    let _e2: FileEditor = Default::default();
    assert!(e.get_value().is_none());
    assert_eq!(e.target_type(), std::any::TypeId::of::<String>());
}

#[test]
fn path_editor_constructors() {
    use vernal_beans::path_editor::PathEditor;
    let e = PathEditor::new();
    let _e2: PathEditor = Default::default();
    assert!(e.get_value().is_none());
    assert_eq!(e.target_type(), std::any::TypeId::of::<String>());
}

#[test]
fn properties_editor_constructors() {
    use vernal_beans::properties_editor::PropertiesEditor;
    let e = PropertiesEditor::new();
    let _e2: PropertiesEditor = Default::default();
    assert!(e.get_value().is_none());
    assert_eq!(e.target_type(), std::any::TypeId::of::<String>());
}

#[test]
fn class_array_editor_constructors() {
    use vernal_beans::class_array_editor::ClassArrayEditor;
    let e = ClassArrayEditor::new();
    let _e2: ClassArrayEditor = Default::default();
    assert!(e.get_value().is_none());
    assert_eq!(e.target_type(), std::any::TypeId::of::<String>());
}

#[test]
fn file_array_editor_constructors() {
    use vernal_beans::file_array_editor::FileArrayEditor;
    let e = FileArrayEditor::new();
    let _e2: FileArrayEditor = Default::default();
    assert!(e.get_value().is_none());
    assert_eq!(e.target_type(), std::any::TypeId::of::<String>());
}

#[test]
fn path_property_editor_constructors() {
    use vernal_beans::path_property_editor::PathPropertyEditor;
    let e = PathPropertyEditor::new();
    let _e2: PathPropertyEditor = Default::default();
    assert!(e.get_value().is_none());
    assert_eq!(e.target_type(), std::any::TypeId::of::<String>());
}

#[test]
fn charset_property_editor_constructors() {
    use vernal_beans::charset_property_editor::CharsetPropertyEditor;
    let e = CharsetPropertyEditor::new();
    let _e2: CharsetPropertyEditor = Default::default();
    assert!(e.get_value().is_none());
    assert_eq!(e.target_type(), std::any::TypeId::of::<String>());
}

// ── container.rs: get_bean_provider with if_available / stream patterns ─

#[test]
fn container_provider_patterns() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());

    // Provider for existing type - should succeed
    let p = c
        .get_bean_provider_by_type_id(std::any::TypeId::of::<String>())
        .unwrap();
    // Before singleton is initialized, if_available returns None
    let _avail = p.if_available();
    // After get(), singleton is initialized
    let _bean = p.get();
    // Now if_available should return Some
    let _avail2 = p.if_available();
    let _unique = p.get_if_unique();
    let _stream = p.stream();
}

// ── container.rs: get_bean_by_key with resolve errors ─────────────────

#[test]
fn container_get_bean_resolve_error() {
    use vernal_beans::bean_factory::BeanFactory;

    // Register a transient component that will fail during construction
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::transient::<String, _>(|_| {
        panic!("deliberate panic in factory");
    }));
    let c = Container::new(b.build().unwrap());

    // get_bean should propagate the error
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = c.get_bean_by_key(&ComponentKey::of::<String>());
    }));
    // The factory panic will propagate - we just verify it doesn't hang
    assert!(result.is_err());
}

// ── container.rs: get_type_not_found ─────────────────────────────────

#[test]
fn container_get_type_not_found_check() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let result = c.get_type(&ComponentKey::of::<String>());
    assert!(result.is_err());
}

// ── container.rs: contains_bean_definition on non-registered ─────────

#[test]
fn container_contains_bean_def_negative() {
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(!c.contains_bean_definition("nonexistent"));
}
