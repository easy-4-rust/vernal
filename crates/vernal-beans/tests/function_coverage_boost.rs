/// 函数覆盖率提升测试 — 针对未覆盖的核心文件方法。
use std::any::Any;
use std::sync::Arc;

fn make_container() -> vernal_beans::Container {
    let mut b = vernal_beans::RegistryBuilder::new();
    b.register(vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    b.register(vernal_beans::ComponentDefinition::singleton::<i32, _>(|_| 42i32));
    vernal_beans::Container::new(b.build().unwrap())
}

// ═══════════════════════════════════════════════════════════════════
// ConstructorArgumentValues 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn cav_value_holder_new() {
    use vernal_beans::factory::config::constructor_argument_values::ValueHolder;
    let vh = ValueHolder::new(Arc::new(42i32));
    assert!(vh.value().is_some());
    assert!(vh.type_name().is_none());
    assert!(vh.name().is_none());
    assert!(!vh.is_converted());
    assert!(vh.converted_value().is_none());
}

#[test]
fn cav_value_holder_with_type() {
    use vernal_beans::factory::config::constructor_argument_values::ValueHolder;
    let vh = ValueHolder::with_type(Arc::new("hello".to_string()), "java.lang.String");
    assert!(vh.value().is_some());
    assert_eq!(vh.type_name(), Some("java.lang.String"));
}

#[test]
fn cav_value_holder_with_type_and_name() {
    use vernal_beans::factory::config::constructor_argument_values::ValueHolder;
    let vh = ValueHolder::with_type_and_name(Arc::new(42i32), "int", "count");
    assert_eq!(vh.type_name(), Some("int"));
    assert_eq!(vh.name(), Some("count"));
}

#[test]
fn cav_value_holder_copy() {
    use vernal_beans::factory::config::constructor_argument_values::ValueHolder;
    let vh = ValueHolder::with_type(Arc::new(42i32), "int");
    let vh2 = vh.copy();
    assert!(vh2.value().is_some());
    assert_eq!(vh2.type_name(), Some("int"));
}

#[test]
fn cav_value_holder_set_converted() {
    use vernal_beans::factory::config::constructor_argument_values::ValueHolder;
    let mut vh = ValueHolder::new(Arc::new("42"));
    vh.set_converted_value(Arc::new(42i32));
    assert!(vh.is_converted());
    assert!(vh.converted_value().is_some());
}

#[test]
fn cav_indexed_args() {
    use vernal_beans::ConstructorArgumentValues;
    use vernal_beans::factory::config::constructor_argument_values::ValueHolder;
    let mut cav = ConstructorArgumentValues::new();
    assert!(cav.is_empty());
    assert_eq!(cav.argument_count(), 0);
    cav.add_indexed_argument_value(0, ValueHolder::new(Arc::new("first")));
    cav.add_indexed_argument_value(1, ValueHolder::new(Arc::new("second")));
    assert!(!cav.is_empty());
    assert_eq!(cav.argument_count(), 2);
    assert!(cav.has_indexed_argument_value(0));
    assert!(!cav.has_indexed_argument_value(99));
    assert!(cav.get_indexed_argument_value(0).is_some());
    assert!(cav.get_indexed_argument_value(99).is_none());
}

#[test]
fn cav_generic_args() {
    use vernal_beans::ConstructorArgumentValues;
    use vernal_beans::factory::config::constructor_argument_values::ValueHolder;
    let mut cav = ConstructorArgumentValues::new();
    cav.add_generic_argument_value(ValueHolder::with_type(Arc::new("val"), "String"));
    assert_eq!(cav.generic_argument_values().len(), 1);
    assert!(cav.get_generic_argument_value("String").is_some());
    assert!(cav.get_generic_argument_value("Integer").is_none());
}

#[test]
fn cav_contains_named() {
    use vernal_beans::ConstructorArgumentValues;
    use vernal_beans::factory::config::constructor_argument_values::ValueHolder;
    let mut cav = ConstructorArgumentValues::new();
    assert!(!cav.contains_named_argument());
    cav.add_generic_argument_value(ValueHolder::with_type_and_name(Arc::new("val"), "String", "name"));
    assert!(cav.contains_named_argument());
}

#[test]
fn cav_from_other() {
    use vernal_beans::ConstructorArgumentValues;
    use vernal_beans::factory::config::constructor_argument_values::ValueHolder;
    let mut original = ConstructorArgumentValues::new();
    original.add_indexed_argument_value(0, ValueHolder::new(Arc::new("val")));
    original.add_generic_argument_value(ValueHolder::with_type(Arc::new("gen"), "String"));
    let copy = ConstructorArgumentValues::from_other(&original);
    assert!(copy.has_indexed_argument_value(0));
    assert_eq!(copy.generic_argument_values().len(), 1);
}

#[test]
fn cav_clear() {
    use vernal_beans::ConstructorArgumentValues;
    use vernal_beans::factory::config::constructor_argument_values::ValueHolder;
    let mut cav = ConstructorArgumentValues::new();
    cav.add_indexed_argument_value(0, ValueHolder::new(Arc::new("val")));
    cav.add_generic_argument_value(ValueHolder::with_type(Arc::new("gen"), "String"));
    assert!(!cav.is_empty());
    cav.clear();
    assert!(cav.is_empty());
}

#[test]
fn cav_add_argument_values() {
    use vernal_beans::ConstructorArgumentValues;
    use vernal_beans::factory::config::constructor_argument_values::ValueHolder;
    let mut cav1 = ConstructorArgumentValues::new();
    cav1.add_indexed_argument_value(0, ValueHolder::new(Arc::new("val1")));
    let mut cav2 = ConstructorArgumentValues::new();
    cav2.add_indexed_argument_value(1, ValueHolder::new(Arc::new("val2")));
    cav2.add_generic_argument_value(ValueHolder::with_type(Arc::new("gen"), "String"));
    cav1.add_argument_values(&cav2);
    assert!(cav1.has_indexed_argument_value(0));
    assert!(cav1.has_indexed_argument_value(1));
    assert_eq!(cav1.generic_argument_values().len(), 1);
}

// ═══════════════════════════════════════════════════════════════════
// GenericBeanDefinition 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn generic_bean_definition_setters() {
    use vernal_beans::GenericBeanDefinition;
    let mut gbd = GenericBeanDefinition::new();
    gbd.set_bean_class_name("com.example.Service");
    gbd.set_parent_name("parentBean");
    gbd.set_scope(vernal_beans::Scope::Transient);
    gbd.set_lazy_init(true);
    gbd.set_abstract(true);
    gbd.set_autowire_candidate(false);
    gbd.set_primary(true);
    gbd.set_fallback(true);
    gbd.set_synthetic(true);
    gbd.set_role(1);
    gbd.set_description("A test service");
    gbd.add_depends_on("dataSource");
    gbd.set_autowire_mode(vernal_beans::Autowire::ByType);
    
    
    
    
}

#[test]
fn generic_bean_definition_from_root() {
    use vernal_beans::GenericBeanDefinition;
    use vernal_beans::RootBeanDefinition;
    let rbd = RootBeanDefinition::new();
    let gbd = GenericBeanDefinition::from_root(&rbd);
    assert_eq!(gbd.scope(), vernal_beans::Scope::Singleton);
}

// ═══════════════════════════════════════════════════════════════════
// TransientTracker 测试
// ═══════════════════════════════════════════════════════════════════

#[test]

#[test]

#[test]

#[test]

// ═══════════════════════════════════════════════════════════════════
// BeanDefinition trait 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bean_definition_trait_methods() {
    use vernal_beans::BeanDefinition;
    use vernal_beans::RootBeanDefinition;
    let mut rbd = RootBeanDefinition::new();
    rbd.set_bean_class_name("com.example.Service");
    rbd.set_scope(vernal_beans::Scope::Transient);
    rbd.set_lazy_init(true);
    rbd.set_abstract(true);
    rbd.set_primary(true);
    rbd.set_fallback(true);
    rbd.set_synthetic(true);
    rbd.set_role(1);
    rbd.set_description("Test description");

    assert_eq!(rbd.bean_class_name(), "com.example.Service");
    assert_eq!(rbd.scope(), vernal_beans::Scope::Transient);
    assert!(rbd.is_lazy_init());
    assert!(rbd.is_abstract());
    assert!(rbd.is_primary());
    assert!(rbd.is_fallback());
    assert!(rbd.is_synthetic());
    assert_eq!(rbd.role(), 1);
    assert_eq!(rbd.description(), Some("Test description"));
}

// ═══════════════════════════════════════════════════════════════════
// InjectionPoint 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn injection_point_basic() {
    use vernal_beans::InjectionPoint;
    let ip = InjectionPoint::new(std::any::TypeId::of::<String>(), "alloc::string::String");
    assert_eq!(ip.type_id(), std::any::TypeId::of::<String>());
    assert_eq!(ip.type_name(), "alloc::string::String");
    assert!(ip.containing_bean_name().is_none());
    assert!(ip.member_name().is_none());
    assert!(ip.qualifier().is_none());
}

#[test]
fn injection_point_with_options() {
    use vernal_beans::InjectionPoint;
    let ip = InjectionPoint::new(std::any::TypeId::of::<i32>(), "i32")
        .with_containing_bean_name("myBean")
        .with_member_name("count")
        .with_qualifier("primary");
    assert_eq!(ip.containing_bean_name(), Some("myBean"));
    assert_eq!(ip.member_name(), Some("count"));
    assert_eq!(ip.qualifier(), Some("primary"));
}

// ═══════════════════════════════════════════════════════════════════
// BeanDefinitionBuilder 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bean_definition_builder_generic_full() {
    use vernal_beans::BeanDefinitionBuilder;
    use vernal_beans::BeanDefinition;
    let def = BeanDefinitionBuilder::generic("com.example.Service")
        .set_parent_name("parentService")
        .set_scope(vernal_beans::Scope::Transient)
        .set_lazy_init(true)
        .set_abstract(true)
        .set_autowire_candidate(false)
        .set_primary(true)
        .set_fallback(true)
        .set_role(1)
        .set_description("A test service")
        .add_depends_on("dataSource")
        .set_init_method("init")
        .set_destroy_method("destroy")
        .add_constructor_arg_value(Arc::new("arg1"))
        .add_constructor_arg_typed(Arc::new(42i32), "int")
        .add_property_value("name", Arc::new("test"))
        .build();
    assert!(def.is_lazy_init());
    assert!(def.is_abstract());
}

#[test]
fn bean_definition_builder_root_full() {
    use vernal_beans::BeanDefinitionBuilder;
    use vernal_beans::BeanDefinition;
    let def = BeanDefinitionBuilder::root("com.example.Root")
        .set_scope(vernal_beans::Scope::Transient)
        .set_lazy_init(true)
        .set_primary(true)
        .set_init_method("init")
        .set_destroy_method("destroy")
        .add_depends_on("dep1")
        .add_constructor_arg_value(Arc::new("arg1"))
        .add_property_value("name", Arc::new("test"))
        .build();
    assert!(def.is_lazy_init());
    assert!(def.is_primary());
}

// ═══════════════════════════════════════════════════════════════════
// BeanWrapperImpl 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bean_wrapper_batch_set() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::bean_wrapper::BeanWrapper;
    use vernal_beans::property_accessor::PropertyAccessor;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    wrapper.register_property("x", std::any::TypeId::of::<i32>());
    wrapper.register_property("y", std::any::TypeId::of::<i32>());
    let mut values = std::collections::HashMap::new();
    values.insert("x".to_string(), Arc::new(10i32) as Arc<dyn Any + Send + Sync>);
    values.insert("y".to_string(), Arc::new(20i32) as Arc<dyn Any + Send + Sync>);
    wrapper.set_property_values(&values).unwrap();
    let x = wrapper.get_property_value("x").unwrap().downcast_ref::<i32>().copied();
    assert_eq!(x, Some(10));
}

#[test]
fn bean_wrapper_wrapped_instance() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::bean_wrapper::BeanWrapper;
    let instance: Arc<dyn Any + Send + Sync> = Arc::new("my_bean".to_string());
    let wrapper = BeanWrapperImpl::new(Arc::clone(&instance));
    assert_eq!(wrapper.get_wrapped_class(), std::any::TypeId::of::<String>());
    let s = wrapper.get_wrapped_instance().downcast_ref::<String>().unwrap();
    assert_eq!(s, "my_bean");
}

// ═══════════════════════════════════════════════════════════════════
// Container 核心方法测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_multiple_types() {
    let c = make_container();
    let s: Arc<String> = c.resolve().unwrap();
    let i: Arc<i32> = c.resolve().unwrap();
    assert_eq!(*s, "hello");
    assert_eq!(*i, 42);
}

#[test]
fn container_resolve_qualified() {
    let mut b = vernal_beans::RegistryBuilder::new();
    let q = vernal_beans::Qualifier::new("primary").unwrap();
    b.register(vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "primary_val".to_string()).qualified(q.clone()));
    let c = vernal_beans::Container::new(b.build().unwrap());
    let val: Arc<String> = c.resolve_qualified(&q).unwrap();
    assert_eq!(*val, "primary_val");
}

#[test]
fn container_warm_up() {
    let mut b = vernal_beans::RegistryBuilder::new();
    b.register(vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    b.register(vernal_beans::ComponentDefinition::singleton::<i32, _>(|_| 42i32));
    let c = vernal_beans::Container::new(b.build().unwrap());
    assert!(c.warm_up().is_ok());
    assert!(c.unused_definitions().is_empty());
}

#[test]
fn container_post_processor() {
    use vernal_beans::BeanPostProcessor;
    let mut c = make_container();
    struct PP; impl BeanPostProcessor for PP {}
    c.add_bean_post_processor(Arc::new(PP));
    c.add_bean_post_processor(Arc::new(PP));
    assert_eq!(c.bean_post_processor_count(), 2);
}

#[test]
fn container_scope() {
    let c = make_container();
    let _scope = c.open_scope::<String>();
}

#[test]
fn container_transient_tracker() {
    let c = make_container();
    let _ = c.transient_tracker();
}
