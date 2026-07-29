//! BeanDefinition hierarchy file tests.
use std::sync::Arc;

// ── AbstractBeanDefinition ─────────────────────────────────────────

#[test]
fn abstract_bean_def_new() {
    use vernal_beans::abstract_bean_definition::AbstractBeanDefinition;
    let d = AbstractBeanDefinition::new();
    assert!(!d.is_lazy_init());
    assert!(!d.is_primary());
    assert_eq!(d.scope(), vernal_beans::Scope::Singleton);
}

#[test]
fn abstract_bean_def_setters() {
    use vernal_beans::abstract_bean_definition::AbstractBeanDefinition;
    let mut d = AbstractBeanDefinition::new();
    d.set_scope(vernal_beans::Scope::Transient);
    d.set_lazy_init(true);
    d.set_primary(true);
    assert!(d.is_lazy_init());
    assert!(d.is_primary());
    assert_eq!(d.scope(), vernal_beans::Scope::Transient);
}

// ── ChildBeanDefinition ─────────────────────────────────────────────

#[test]
fn child_bean_def_new() {
    use vernal_beans::child_bean_definition::ChildBeanDefinition;
    let d = ChildBeanDefinition::new("parentBean");
    assert_eq!(d.parent_name(), "parentBean");
}

// ── AnnotatedGenericBeanDefinition ─────────────────────────────────

#[test]
fn annotated_generic_bean_def_new() {
    use vernal_beans::annotated_generic_bean_definition::AnnotatedGenericBeanDefinition;
    use vernal_beans::bean_definition::BeanDefinition;
    let def = AnnotatedGenericBeanDefinition::new("com.example.Config");
    assert_eq!(def.bean_class_name(), "unknown");
    assert_eq!(def.scope(), vernal_beans::Scope::Singleton);
}

// ── BeanDefinitionHolder ───────────────────────────────────────────

#[test]
fn bean_def_holder_new() {
    use vernal_beans::bean_definition_holder::BeanDefinitionHolder;
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    let def = Arc::new(RootBeanDefinition::new()) as Arc<dyn vernal_beans::BeanDefinition>;
    let holder = BeanDefinitionHolder::new("myBean", def);
    assert_eq!(holder.bean_name(), "myBean");
}

// ── SimpleBeanDefinitionRegistry ──────────────────────────────────

#[test]
fn simple_registry_basic() {
    use vernal_beans::simple_bean_definition_registry::SimpleBeanDefinitionRegistry;
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
    use vernal_beans::root_bean_definition::RootBeanDefinition;

    let mut registry = SimpleBeanDefinitionRegistry::new();
    let def = Box::new(RootBeanDefinition::new());
    registry.register_bean_definition("test".to_string(), def).unwrap();
    assert!(registry.contains_bean_definition("test"));
    assert_eq!(registry.bean_definition_count(), 1);
}

// ── StaticListableBeanFactory ─────────────────────────────────────

#[test]
fn static_factory_basic() {
    use vernal_beans::static_listable_bean_factory::StaticListableBeanFactory;
    let mut f = StaticListableBeanFactory::new();
    f.register_singleton("bean1", Arc::new(42i32));
    assert!(f.contains_bean_name("bean1"));
    assert_eq!(f.bean_names(), vec!["bean1".to_string()]);
}

// ── InjectedElement ───────────────────────────────────────────────

#[test]
fn injected_element_new() {
    use vernal_beans::injected_element::InjectedElement;
    let e = InjectedElement::new("name", true);
    assert_eq!(e.get_name(), "name");
    assert!(e.is_required());
}

// ── InjectionMetadata ─────────────────────────────────────────────

#[test]
fn injection_metadata_basic() {
    use vernal_beans::injection_metadata::InjectionMetadata;
    let mut m = InjectionMetadata::new();
    assert!(m.is_empty());
    m.add_element(Box::new("item".to_string()));
    assert_eq!(m.element_count(), 1);
    m.clear();
    assert!(m.is_empty());
}

// ── AutowiredFieldElement ─────────────────────────────────────────

#[test]
fn autowired_field_element_new() {
    use vernal_beans::autowired_field_element::AutowiredFieldElement;
    let e = AutowiredFieldElement::new("myField", true);
    assert_eq!(e.get_name(), "myField");
    assert!(e.is_required());
}

// ── AutowiredMethodElement ─────────────────────────────────────────

#[test]
fn autowired_method_element_new() {
    use vernal_beans::autowired_method_element::AutowiredMethodElement;
    let e = AutowiredMethodElement::new("myMethod", vec![std::any::TypeId::of::<String>()]);
    assert_eq!(e.get_method_name(), "myMethod");
    assert_eq!(e.get_parameter_type_ids().len(), 1);
}

// ── Managed collections ────────────────────────────────────────────

#[test]
fn managed_list_basic() {
    use vernal_beans::managed_list::ManagedList;
    let mut l = ManagedList::new();
    assert!(l.is_empty());
    l.push(42);
    assert_eq!(l.len(), 1);
    let v = l.into_vec();
    assert_eq!(v, vec![42]);
}

#[test]
fn managed_set_basic() {
    use vernal_beans::managed_set::ManagedSet;
    let mut s = ManagedSet::new();
    s.insert("hello".to_string());
    assert!(s.contains(&"hello".to_string()));
    assert_eq!(s.len(), 1);
}

#[test]
fn managed_map_basic() {
    use vernal_beans::managed_map::ManagedMap;
    let mut m = ManagedMap::new();
    m.insert("key1", "value1");
    assert!(m.contains_key(&"key1"));
    assert_eq!(m.get(&"key1"), Some(&"value1"));
    assert_eq!(m.len(), 1);
}

#[test]
fn managed_properties_basic() {
    use vernal_beans::managed_properties::ManagedProperties;
    let mut p = ManagedProperties::new();
    p.set("name", "value");
    assert_eq!(p.get("name"), Some("value"));
    assert!(p.contains_key("name"));
    assert_eq!(p.len(), 1);
}

#[test]
fn managed_array_basic() {
    use vernal_beans::managed_array::ManagedArray;
    let mut a = ManagedArray::new();
    a.push(1);
    a.push(2);
    assert_eq!(a.len(), 2);
    let v = a.into_vec();
    assert_eq!(v, vec![1, 2]);
}

// ── Method overrides ────────────────────────────────────────────────

#[test]
fn method_override_basic() {
    use vernal_beans::method_override::{SimpleMethodOverride, MethodOverride};
    let o = SimpleMethodOverride::new("doSomething");
    assert_eq!(o.get_method_name(), "doSomething");
    assert!(o.is_applicable());
}

#[test]
fn method_overrides_basic() {
    use vernal_beans::method_overrides::MethodOverrides;
    use vernal_beans::method_override::{SimpleMethodOverride, MethodOverride};
    let mut mo = MethodOverrides::new();
    mo.add("doSomething".to_string(), Box::new(SimpleMethodOverride::new("doSomething")));
    assert_eq!(mo.len(), 1);
    assert!(!mo.is_empty());
    assert!(mo.get("doSomething").is_some());
}

#[test]
fn lookup_override_basic() {
    use vernal_beans::lookup_override::LookupOverride;
    use vernal_beans::method_override::MethodOverride;
    let o = LookupOverride::new("createBean", "myBean");
    assert_eq!(o.get_method_name(), "createBean");
    assert_eq!(o.get_bean_name(), "myBean");
    assert!(o.is_applicable());
}

#[test]
fn replace_override_basic() {
    use vernal_beans::replace_override::ReplaceOverride;
    use vernal_beans::method_override::MethodOverride;
    let o = ReplaceOverride::new("doSomething", "myReplacer");
    assert_eq!(o.get_method_name(), "doSomething");
    assert_eq!(o.get_replacer_name(), "myReplacer");
    assert!(o.is_applicable());
}

// ── MethodDescriptor ───────────────────────────────────────────────

#[test]
fn method_descriptor_basic() {
    use vernal_beans::method_descriptor::MethodDescriptor;
    let d = MethodDescriptor::new("myMethod").with_return_type("String").with_parameters(vec!["i32".to_string()]);
    assert_eq!(d.get_name(), "myMethod");
    assert_eq!(d.get_return_type(), "String");
    assert_eq!(d.parameter_count(), 1);
}

// ── CustomEditorConfigurer ────────────────────────────────────────

#[test]
fn custom_editor_configurer_basic() {
    use vernal_beans::custom_editor_configurer::CustomEditorConfigurer;
    use vernal_beans::class_editor::ClassEditor;
    let mut c = CustomEditorConfigurer::new();
    c.register_custom_editor(std::any::TypeId::of::<String>(), Box::new(ClassEditor::new()));
    assert!(c.has_custom_editor(std::any::TypeId::of::<String>()));
    assert!(!c.has_custom_editor(std::any::TypeId::of::<i32>()));
    assert_eq!(c.editor_count(), 1);
}

// ── BeanWiringInfo ─────────────────────────────────────────────────

#[test]
fn bean_wiring_info_basic() {
    use vernal_beans::bean_wiring_info::BeanWiringInfo;
    let info = BeanWiringInfo::new("myBean", "java.lang.String");
    assert_eq!(info.get_bean_name(), "myBean");
    assert_eq!(info.get_type_name(), "java.lang.String");
    assert!(info.is_default_dependency());
    let s = format!("{}", info);
    assert!(s.contains("myBean"));
}
