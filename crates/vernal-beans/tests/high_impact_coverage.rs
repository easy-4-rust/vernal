//! High-impact coverage tests for 0%-covered and heavily uncovered files.
use std::sync::Arc;
use vernal_beans::{
    bean_definition::BeanDefinition, bean_factory::BeanFactory,
    bean_definition_registry::BeanDefinitionRegistry,
    autowire_capable_bean_factory::AutowireCapableBeanFactory,
    ComponentDefinition, ComponentKey, Container, RegistryBuilder,
};

// ── mutable_property_sources.rs (32 missed, 0%) ─────────────────

#[test]
fn mutable_property_sources_all_methods() {
    use vernal_beans::mutable_property_sources::MutablePropertySources;
    use vernal_beans::property_source::PropertySource;
    
    let mut mps = MutablePropertySources::new();
    assert!(mps.as_inner().is_empty());
    
    let mut ps1 = PropertySource::new("env");
    ps1.set("KEY", "VALUE");
    mps.add_first(ps1);
    
    let mut ps2 = PropertySource::new("props");
    ps2.set("k", "v");
    mps.add_last(ps2);
    assert_eq!(mps.as_inner().len(), 2);
    
    let ps3 = PropertySource::new("between");
    mps.add_before("props", ps3);
    assert_eq!(mps.as_inner().len(), 3);
    
    let ps4 = PropertySource::new("after");
    mps.add_after("env", ps4);
    assert_eq!(mps.as_inner().len(), 4);
    
    let ps5 = PropertySource::new("env");
    mps.replace("env", ps5);
    
    mps.remove("env");
    assert_eq!(mps.as_inner().len(), 3);
}

// ── static_listable_bean_factory.rs (25 missed, 0%) ─────────────

#[test]
fn static_factory_all_methods() {
    use vernal_beans::static_listable_bean_factory::StaticListableBeanFactory;
    
    let mut f = StaticListableBeanFactory::new();
    f.register_singleton("bean1", Arc::new(42i32));
    f.register_singleton("bean2", Arc::new("hello".to_string()));
    
    assert!(f.contains_bean_name("bean1"));
    assert!(!f.contains_bean_name("nonexistent"));
    assert_eq!(f.bean_names().len(), 2);
}

// ── simple_bean_definition_registry.rs (23 missed, 0%) ──────────

#[test]
fn simple_registry_all_methods() {
    use vernal_beans::simple_bean_definition_registry::SimpleBeanDefinitionRegistry;
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    
    let mut registry = SimpleBeanDefinitionRegistry::new();
    let def = Box::new(RootBeanDefinition::new());
    registry.register_bean_definition("test".to_string(), def).unwrap();
    assert!(registry.contains_bean_definition("test"));
    assert_eq!(registry.bean_definition_count(), 1);
    
    let names = registry.bean_definition_names();
    assert!(names.contains(&"test".to_string()));
    
    registry.clear();
    assert!(!registry.contains_bean_definition("test"));
}

// ── custom_editor_configurer.rs (16 missed, 0%) ──────────────────

#[test]
fn configurer_all_methods() {
    use vernal_beans::custom_editor_configurer::CustomEditorConfigurer;
    use vernal_beans::class_editor::ClassEditor;
    
    let mut c = CustomEditorConfigurer::new();
    assert_eq!(c.editor_count(), 0);
    assert!(!c.has_custom_editor(std::any::TypeId::of::<String>()));
    
    c.register_custom_editor(std::any::TypeId::of::<String>(), Box::new(ClassEditor::new()));
    assert!(c.has_custom_editor(std::any::TypeId::of::<String>()));
    assert_eq!(c.editor_count(), 1);
    assert!(c.get_custom_editor(std::any::TypeId::of::<String>()).is_some());
    assert!(c.get_custom_editor(std::any::TypeId::of::<i32>()).is_none());
}

// ── abstract_bean_definition.rs (20 missed, 51.22%) ─────────────

#[test]
fn abstract_bean_def_all_methods() {
    use vernal_beans::abstract_bean_definition::AbstractBeanDefinition;
    
    let mut def = AbstractBeanDefinition::new();
    assert_eq!(def.scope(), vernal_beans::Scope::Singleton);
    assert!(def.bean_class_name().is_none());
    assert!(!def.is_lazy_init());
    assert!(!def.is_primary());
    
    def.set_scope(vernal_beans::Scope::Transient);
    def.set_lazy_init(true);
    def.set_primary(true);
    assert_eq!(def.scope(), vernal_beans::Scope::Transient);
    assert!(def.is_lazy_init());
    assert!(def.is_primary());
}

// ── bean_definition.rs (24 missed, 46.67%) ──────────────────────

#[test]
fn bean_definition_trait_methods() {
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    
    let def = RootBeanDefinition::new();
    let bd: &dyn BeanDefinition = &def;
    assert_eq!(bd.bean_class_name(), "unknown");
    assert_eq!(bd.scope(), vernal_beans::Scope::Singleton);
    assert!(!bd.is_lazy_init());
    assert!(!bd.is_primary());
    assert!(!bd.is_fallback());
    assert!(bd.is_autowire_candidate());
    assert_eq!(bd.role(), 0);
    assert!(bd.description().is_none());
    assert!(bd.parent_name().is_none());
    assert!(bd.factory_bean_name().is_none());
    assert!(bd.factory_method_name().is_none());
    assert!(bd.init_method_name().is_none());
    assert!(bd.destroy_method_name().is_none());
    assert!(!bd.is_abstract());
    assert!(bd.is_singleton());
    assert!(!bd.is_prototype());
    assert!(bd.resource_description().is_none());
    assert!(bd.originating_bean_definition().is_none());
}

// ── root_bean_definition.rs (38 missed, 82.57%) ─────────────────

#[test]
fn root_bean_def_setters_and_getters() {
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    
    let mut def = RootBeanDefinition::new();
    def.set_bean_class_name("com.example.Test");
    def.set_scope(vernal_beans::Scope::Transient);
    def.set_lazy_init(true);
    def.set_primary(true);
    def.set_autowire_candidate(false);
    def.set_role(2);
    def.set_description("desc");
    def.add_depends_on("dep1");
    def.set_init_method_name("init");
    def.set_destroy_method_name("destroy");
    def.set_factory_bean_name("fb");
    def.set_factory_method_name("fm");
    def.set_abstract(true);
    def.set_synthetic(true);
    def.set_fallback(true);
    
    assert_eq!(def.bean_class_name(), "com.example.Test");
    assert_eq!(def.scope(), vernal_beans::Scope::Transient);
    assert!(def.is_lazy_init());
    assert!(def.is_primary());
    assert!(!def.is_autowire_candidate());
    assert_eq!(def.role(), 2);
    assert_eq!(def.description(), Some("desc"));
    assert_eq!(def.depends_on(), &["dep1"]);
    assert_eq!(def.autowire_mode(), vernal_beans::Autowire::No);
    assert_eq!(def.init_method_name(), Some("init"));
    assert_eq!(def.destroy_method_name(), Some("destroy"));
    assert_eq!(def.factory_bean_name(), Some("fb"));
    assert_eq!(def.factory_method_name(), Some("fm"));
    assert!(def.is_abstract());
    assert!(def.is_synthetic());
    assert!(def.is_fallback());
    assert!(def.parent_name().is_none());
    assert!(def.constructor_argument_values().is_empty());
    assert!(def.property_values().is_empty());
}

#[test]
fn root_bean_def_from_generic() {
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    use vernal_beans::generic_bean_definition::GenericBeanDefinition;
    
    let mut generic = GenericBeanDefinition::new();
    generic.set_bean_class_name("com.example.FromGeneric");
    generic.set_scope(vernal_beans::Scope::Transient);
    
    let root = RootBeanDefinition::from(generic);
    assert_eq!(root.bean_class_name(), "com.example.FromGeneric");
    assert_eq!(root.scope(), vernal_beans::Scope::Transient);
}

#[test]
fn root_bean_def_default() {
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    let def: RootBeanDefinition = Default::default();
    assert_eq!(def.bean_class_name(), "unknown");
}

// ── scope_context.rs (20 missed, 93.08%) ─────────────────────────

#[test]
fn scope_context_open_close() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let scope = c.open_scope::<String>();
    let _key = scope.key();
}

// ── registry_builder.rs (26 missed, 89.88%) ─────────────────────

#[test]
fn registry_builder_register_and_get() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    assert!(b.contains::<String>());
    assert_eq!(b.len(), 1);
    assert!(!b.is_empty());
    let _r = b.build().unwrap();
}

#[test]
fn registry_builder_register_all() {
    let mut b = RegistryBuilder::new();
    b.register_all(vec![
        ComponentDefinition::singleton::<String, _>(|_| "a".to_string()),
        ComponentDefinition::singleton::<i32, _>(|_| 42i32),
    ]);
    assert_eq!(b.len(), 2);
}

// ── standard_bean_expression_resolver.rs (17 missed, 80%) ────────

#[test]
fn expression_resolver_simple() {
    use vernal_beans::bean_expression_resolver::BeanExpressionResolver;
    use vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver;
    let r = StandardBeanExpressionResolver::new();
    let _ = r.evaluate("myBean", None);
}

// ── component_definition.rs (18 missed, 90.53%) ──────────────────

#[test]
fn component_def_key_scope_deps() {
    let def = ComponentDefinition::singleton::<String, _>(|_| "hello".to_string());
    assert_eq!(def.key().type_name(), std::any::type_name::<String>());
    assert!(def.scope().is_singleton());
    assert!(def.dependencies().is_empty());
}

// ── dependency_descriptor.rs (24 missed, 67.57%) ─────────────────

#[test]
fn dependency_descriptor_methods() {
    use vernal_beans::DependencyDescriptor;
    
    let desc = DependencyDescriptor::for_field(
        std::any::TypeId::of::<String>(),
        "String",
    );
    assert_eq!(desc.type_id(), std::any::TypeId::of::<String>());
    assert_eq!(desc.type_name(), "String");
    assert!(desc.qualifier().is_none());
    assert!(!desc.is_optional());
    
    let desc2 = desc.with_optional(true);
    assert!(desc2.is_optional());
    
    let desc3 = DependencyDescriptor::for_constructor_parameter(
        0,
        std::any::TypeId::of::<i32>(),
        "i32",
    );
    assert_eq!(desc3.field_index(), Some(0));
}
