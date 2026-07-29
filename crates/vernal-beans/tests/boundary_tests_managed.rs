//! 边界测试：覆盖 20 个 Spring 风格模块中未覆盖的方法与分支。
//!
//! 本测试文件聚焦于各个受管类型、注入元素、方法覆盖、注册表与配置器
//! 的边界条件与错误路径。所有断言均针对真实公开 API。

use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use vernal_beans::RootBeanDefinition;
use vernal_beans::autowired_field_element::AutowiredFieldElement;
use vernal_beans::autowired_method_element::AutowiredMethodElement;
use vernal_beans::bean_definition::BeanDefinition;
use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
use vernal_beans::bean_wiring_info::BeanWiringInfo;
use vernal_beans::custom_autowire_configurer::{
    CustomAutowireConfigurer, NoopCustomAutowireConfigurer,
};
use vernal_beans::custom_editor_configurer::CustomEditorConfigurer;
use vernal_beans::injected_element::InjectedElement;
use vernal_beans::injection_metadata::InjectionMetadata;
use vernal_beans::injection_point::InjectionPoint;
use vernal_beans::lookup_override::LookupOverride;
use vernal_beans::managed_array::ManagedArray;
use vernal_beans::managed_list::ManagedList;
use vernal_beans::managed_map::ManagedMap;
use vernal_beans::managed_properties::ManagedProperties;
use vernal_beans::managed_set::ManagedSet;
use vernal_beans::method_descriptor::MethodDescriptor;
use vernal_beans::method_override::{GenericMethodOverride, MethodOverride, MethodOverrideBase};
use vernal_beans::method_overrides::MethodOverrides;
use vernal_beans::method_replacer::MethodReplacer;
use vernal_beans::property_editor::PropertyEditor;
use vernal_beans::replace_override::ReplaceOverride;
use vernal_beans::simple_bean_definition_registry::SimpleBeanDefinitionRegistry;
use vernal_beans::static_listable_bean_factory::StaticListableBeanFactory;

// =============================================================================
// 测试辅助：构造一个可作为 BeanDefinition 注册的类型擦除定义。
// =============================================================================

/// 用 `RootBeanDefinition` 作为最简 BeanDefinition 来源（实现了该 trait）。
fn boxed_definition() -> Box<dyn BeanDefinition> {
    Box::new(RootBeanDefinition::new())
}

/// 构造一个用于 `&dyn BeanDefinition` 场景的定义（Arc 共享）。
fn shared_definition() -> Arc<dyn BeanDefinition> {
    Arc::new(RootBeanDefinition::new())
}

// =============================================================================
// 1. InjectedElement
// =============================================================================

#[test]
fn injected_element_default_member_name_is_empty() {
    let el = InjectedElement::default();
    assert_eq!(el.member_name(), "");
    assert!(!el.is_required());
    assert!(!el.is_injected());
}

#[test]
fn injected_element_custom_member_name_and_set_required() {
    let mut el = InjectedElement::new("myField").with_required(true);
    assert_eq!(el.member_name(), "myField");
    assert!(el.is_required());

    // set_required 切换标记
    el.set_required(false);
    assert!(!el.is_required());
    el.set_required(true);
    assert!(el.is_required());
}

#[test]
fn injected_element_inject_marks_injected_and_is_idempotent() {
    let mut el = InjectedElement::new("f");
    assert!(!el.is_injected());

    let mut target = 0i32;
    let res = el.inject(&mut target as &mut dyn Any, None);
    assert!(res.is_ok());
    assert!(el.is_injected());

    // 重复注入直接返回 Ok 且不改变状态
    let res2 = el.inject(&mut target as &mut dyn Any, None);
    assert!(res2.is_ok());
    assert!(el.is_injected());

    // clear_injected 复位
    el.clear_injected();
    assert!(!el.is_injected());
}

#[test]
fn injected_element_inject_accepts_some_value() {
    let mut el = InjectedElement::new("g");
    let mut target = 7u32;
    let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    let res = el.inject(&mut target as &mut dyn Any, Some(value));
    assert!(res.is_ok());
    assert!(el.is_injected());
}

// =============================================================================
// 2. InjectionMetadata
// =============================================================================

#[test]
fn injection_metadata_new_is_empty_and_tracks_class() {
    let md = InjectionMetadata::new("com.example.Foo");
    assert_eq!(md.target_class(), "com.example.Foo");
    assert!(md.is_empty());
    assert_eq!(md.len(), 0);
}

#[test]
fn injection_metadata_add_injected_element_increments_len() {
    let mut md = InjectionMetadata::new("Foo");
    md.add_injected_element(InjectedElement::new("a"));
    md.add_injected_element(InjectedElement::new("b"));
    assert!(!md.is_empty());
    assert_eq!(md.len(), 2);
    assert_eq!(md.injected_elements().len(), 2);
    assert_eq!(md.injected_elements()[0].member_name(), "a");
}

#[test]
fn injection_metadata_inject_runs_all_elements_then_clear_resets() {
    let mut md = InjectionMetadata::new("Foo");
    md.add_injected_element(InjectedElement::new("a"));
    md.add_injected_element(InjectedElement::new("b"));

    let mut target = 0i32;
    // inject 应将所有元素标记为已注入
    md.inject(&mut target as &mut dyn Any, None).unwrap();
    for el in md.injected_elements() {
        assert!(el.is_injected());
    }

    // clear 之后所有元素恢复未注入
    md.clear();
    for el in md.injected_elements() {
        assert!(!el.is_injected());
    }
}

#[test]
fn injection_metadata_supports_injection_points() {
    let mut md = InjectionMetadata::new("Bar");
    let pt = InjectionPoint::new(TypeId::of::<u32>(), "u32");
    md.add_injection_point(pt);
    assert_eq!(md.injection_points().len(), 1);
    assert_eq!(md.len(), 1);
    assert!(!md.is_empty());
}

// =============================================================================
// 3. AutowiredFieldElement
// =============================================================================

#[test]
fn autowired_field_resolve_required_missing_returns_error() {
    let el = AutowiredFieldElement::new::<u32>("count", true);
    // resolver 返回 None，且 required=true → 错误路径
    let res = el.resolve(|_| None);
    assert!(res.is_err());
    let err = res.unwrap_err().to_string();
    assert!(err.contains("count"), "unexpected error: {err}");
}

#[test]
fn autowired_field_resolve_optional_missing_returns_none() {
    let el = AutowiredFieldElement::new::<u32>("count", false);
    let res = el.resolve(|_| None).unwrap();
    assert!(res.is_none());
}

#[test]
fn autowired_field_resolve_with_value_returns_some() {
    let el = AutowiredFieldElement::new::<u32>("count", true);
    let v: Arc<dyn Any + Send + Sync> = Arc::new(5u32);
    let got = el.resolve(|_| Some(v)).unwrap();
    assert!(got.is_some());
}

#[test]
fn autowired_field_type_id_and_metadata() {
    let el = AutowiredFieldElement::new::<String>("name", false);
    assert_eq!(el.field_type_id(), TypeId::of::<String>());
    assert_eq!(el.field_name(), "name");
    assert!(!el.required());
    // 基类可访问
    assert_eq!(el.base().member_name(), "name");
    assert!(!el.base().is_required());
}

#[test]
fn autowired_field_with_type_id_constructor() {
    let el = AutowiredFieldElement::with_type_id("x", TypeId::of::<bool>(), true);
    assert_eq!(el.field_type_id(), TypeId::of::<bool>());
    assert!(el.required());
}

// =============================================================================
// 4. AutowiredMethodElement
// =============================================================================

#[test]
fn autowired_method_invoke_collects_arguments() {
    let el = AutowiredMethodElement::new(
        "configure",
        vec![TypeId::of::<u32>(), TypeId::of::<String>()],
        true,
    );
    let args = el
        .invoke(|tid| {
            if tid == TypeId::of::<u32>() {
                Some(Arc::new(1u32) as Arc<dyn Any + Send + Sync>)
            } else if tid == TypeId::of::<String>() {
                Some(Arc::new("s".to_string()) as Arc<dyn Any + Send + Sync>)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(args.len(), 2);
}

#[test]
fn autowired_method_invoke_required_missing_errors() {
    let el = AutowiredMethodElement::new("configure", vec![TypeId::of::<u32>()], true);
    let res = el.invoke(|_| None);
    assert!(res.is_err());
    let err = res.unwrap_err().to_string();
    assert!(err.contains("configure"), "unexpected: {err}");
}

#[test]
fn autowired_method_invoke_optional_missing_skips() {
    let el = AutowiredMethodElement::new("configure", vec![TypeId::of::<u32>()], false);
    let args = el.invoke(|_| None).unwrap();
    assert!(args.is_empty());
}

#[test]
fn autowired_method_metadata_accessors() {
    let el = AutowiredMethodElement::new("setX", vec![TypeId::of::<u32>()], true);
    assert_eq!(el.method_name(), "setX");
    assert_eq!(el.parameter_type_ids(), &[TypeId::of::<u32>()]);
    assert_eq!(el.parameter_count(), 1);
    assert!(el.required());
    assert_eq!(el.base().member_name(), "setX");
}

#[test]
fn autowired_method_base_mut_allows_required_change() {
    let mut el = AutowiredMethodElement::new("m", vec![], false);
    el.base_mut().set_required(true);
    assert!(el.base().is_required());
}

// =============================================================================
// 5. CustomAutowireConfigurer
// =============================================================================

#[test]
fn noop_custom_autowire_configurer_returns_false() {
    let cfg = NoopCustomAutowireConfigurer;
    let def = shared_definition();
    assert!(!cfg.is_custom_autowire_candidate(&*def));
}

#[test]
fn custom_autowire_configurer_default_trait_method_is_false() {
    // 直接调用 trait 默认实现（与 Noop 等价）。
    struct AlwaysNoop;
    impl CustomAutowireConfigurer for AlwaysNoop {}

    let cfg = AlwaysNoop;
    let def = shared_definition();
    assert!(!cfg.is_custom_autowire_candidate(&*def));
}

#[test]
fn custom_autowire_configurer_can_be_overridden_to_true() {
    struct AlwaysCandidate;
    impl CustomAutowireConfigurer for AlwaysCandidate {
        fn is_custom_autowire_candidate(&self, _d: &dyn BeanDefinition) -> bool {
            true
        }
    }
    let cfg = AlwaysCandidate;
    let def = shared_definition();
    assert!(cfg.is_custom_autowire_candidate(&*def));
}

// =============================================================================
// 6. SimpleBeanDefinitionRegistry（全部方法）
// =============================================================================

#[test]
fn simple_registry_lifecycle_register_get_remove() {
    let mut reg = SimpleBeanDefinitionRegistry::new();
    assert_eq!(reg.bean_definition_count(), 0);
    assert!(reg.bean_definition_names().is_empty());

    reg.register_bean_definition("a".to_string(), boxed_definition())
        .unwrap();
    reg.register_bean_definition("b".to_string(), boxed_definition())
        .unwrap();
    assert_eq!(reg.bean_definition_count(), 2);
    assert!(reg.contains_bean_definition("a"));
    assert!(reg.contains_bean_definition("b"));
    assert!(reg.get_bean_definition("a").is_some());
}

#[test]
fn simple_registry_duplicate_registration_overwrites() {
    let mut reg = SimpleBeanDefinitionRegistry::new();
    reg.register_bean_definition("a".to_string(), boxed_definition())
        .unwrap();
    reg.register_bean_definition("a".to_string(), boxed_definition())
        .unwrap();
    // 同名覆盖，计数仍为 1
    assert_eq!(reg.bean_definition_count(), 1);
}

#[test]
fn simple_registry_empty_name_errors() {
    let mut reg = SimpleBeanDefinitionRegistry::new();
    let res = reg.register_bean_definition(String::new(), boxed_definition());
    assert!(res.is_err());
}

#[test]
fn simple_registry_remove_existing_returns_definition() {
    let mut reg = SimpleBeanDefinitionRegistry::new();
    reg.register_bean_definition("a".to_string(), boxed_definition())
        .unwrap();
    let removed = reg.remove_bean_definition("a").unwrap();
    assert_eq!(removed.bean_class_name(), removed.bean_class_name());
    assert!(!reg.contains_bean_definition("a"));
    assert_eq!(reg.bean_definition_count(), 0);
}

#[test]
fn simple_registry_remove_nonexistent_errors() {
    let mut reg = SimpleBeanDefinitionRegistry::new();
    let res = reg.remove_bean_definition("missing");
    assert!(res.is_err());
}

#[test]
fn simple_registry_names_contains_all_keys() {
    let mut reg = SimpleBeanDefinitionRegistry::new();
    reg.register_bean_definition("alpha".to_string(), boxed_definition())
        .unwrap();
    reg.register_bean_definition("beta".to_string(), boxed_definition())
        .unwrap();
    let names = reg.bean_definition_names();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"alpha".to_string()));
    assert!(names.contains(&"beta".to_string()));
}

#[test]
fn simple_registry_clear_empties() {
    let mut reg = SimpleBeanDefinitionRegistry::new();
    reg.register_bean_definition("a".to_string(), boxed_definition())
        .unwrap();
    reg.clear();
    assert_eq!(reg.bean_definition_count(), 0);
    assert!(reg.bean_definition_names().is_empty());
}

// =============================================================================
// 7. StaticListableBeanFactory
// =============================================================================

#[test]
fn static_factory_register_multiple_and_query() {
    let mut factory = StaticListableBeanFactory::new();
    assert!(factory.is_empty());
    factory.register_singleton("a", 1i32);
    factory.register_singleton("b", 2i32);
    factory.register_singleton("c", 3i32);
    assert_eq!(factory.bean_count(), 3);
    assert!(!factory.is_empty());
}

#[test]
fn static_factory_get_bean_not_found_errors() {
    let factory = StaticListableBeanFactory::new();
    let res = factory.get_bean("missing");
    assert!(res.is_err());
    let err = res.unwrap_err().to_string();
    assert!(err.contains("missing"));
}

#[test]
fn static_factory_contains_false_for_missing() {
    let factory = StaticListableBeanFactory::new();
    assert!(!factory.contains_bean("nope"));
    assert!(factory.bean_names().is_empty());
}

#[test]
fn static_factory_bean_names_round_trip() {
    let mut factory = StaticListableBeanFactory::new();
    factory.register_singleton("x", 1u32);
    factory.register_singleton("y", 2u32);
    let names = factory.bean_names();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"x".to_string()));
    assert!(names.contains(&"y".to_string()));
}

#[test]
fn static_factory_typed_get_and_type_filtering() {
    let mut factory = StaticListableBeanFactory::new();
    factory.register_singleton("n", 42i32);
    let typed = factory.get_typed_bean::<i32>("n").unwrap();
    assert_eq!(*typed, 42);

    // 类型不匹配应报错
    let bad = factory.get_typed_bean::<u32>("n");
    assert!(bad.is_err());

    // 按类型查找名称
    let names = factory.bean_names_for_type(TypeId::of::<i32>());
    assert_eq!(names, vec!["n".to_string()]);
}

#[test]
fn static_factory_dyn_register_remove_clear() {
    let mut factory = StaticListableBeanFactory::new();
    let v: Arc<dyn Any + Send + Sync> = Arc::new(true);
    factory.register_singleton_dyn("flag", v, TypeId::of::<bool>(), "bool");
    assert!(factory.contains_bean("flag"));
    assert_eq!(factory.get_type("flag"), Some("bool"));
    assert_eq!(factory.get_type_id("flag"), Some(TypeId::of::<bool>()));

    assert!(factory.remove_bean("flag"));
    assert!(!factory.contains_bean("flag"));
    assert!(!factory.remove_bean("flag"));

    factory.register_singleton("z", 1u8);
    factory.clear();
    assert_eq!(factory.bean_count(), 0);
}

// =============================================================================
// 8. ManagedList
// =============================================================================

#[test]
fn managed_list_from_vec_and_into_vec() {
    let list = ManagedList::from_vec(vec![1, 2, 3]);
    assert_eq!(list.len(), 3);
    assert_eq!(list.as_slice(), &[1, 2, 3]);
    let v = list.into_vec();
    assert_eq!(v, vec![1, 2, 3]);
}

#[test]
fn managed_list_push_and_source_name() {
    let mut list = ManagedList::<i32>::new();
    assert!(list.is_empty());
    list.push(10);
    list.push(20);
    assert_eq!(list.len(), 2);

    let with_source = list.with_source_name("beans.xml");
    assert_eq!(with_source.source_name(), Some("beans.xml"));
    let _ = with_source.into_vec();
}

#[test]
fn managed_list_builder_setters() {
    let list = ManagedList::from_vec(vec!["a"])
        .with_source_name("src")
        .with_element_type_name("String");
    assert_eq!(list.source_name(), Some("src"));
    assert_eq!(list.element_type_name(), Some("String"));
}

#[test]
fn managed_list_from_into_conversions() {
    let list: ManagedList<i32> = vec![1, 2].into();
    let back: Vec<i32> = list.into();
    assert_eq!(back, vec![1, 2]);
}

// =============================================================================
// 9. ManagedSet
// =============================================================================

#[test]
fn managed_set_from_set_and_into_set() {
    let mut hs = HashSet::new();
    hs.insert(1);
    hs.insert(2);
    let set = ManagedSet::from_set(hs);
    assert_eq!(set.len(), 2);
    let back = set.into_set();
    assert_eq!(back.len(), 2);
}

#[test]
fn managed_set_insert_contains_remove() {
    let mut set = ManagedSet::<&str>::new();
    assert!(set.is_empty());
    assert!(set.insert("a"));
    assert!(!set.insert("a")); // 重复插入返回 false
    set.insert("b");
    assert!(set.contains(&"a"));
    assert!(set.contains(&"b"));
    assert!(!set.contains(&"c"));
    assert_eq!(set.len(), 2);

    assert!(set.remove(&"a"));
    assert!(!set.remove(&"a"));
    assert_eq!(set.len(), 1);
}

#[test]
fn managed_set_source_name_and_type_name() {
    let set = ManagedSet::<u32>::new()
        .with_source_name("cfg.xml")
        .with_element_type_name("u32");
    assert_eq!(set.source_name(), Some("cfg.xml"));
    assert_eq!(set.element_type_name(), Some("u32"));
}

#[test]
fn managed_set_conversions() {
    let mut hs = HashSet::new();
    hs.insert(5u8);
    let managed: ManagedSet<u8> = hs.into();
    let back: HashSet<u8> = managed.into();
    assert_eq!(back.len(), 1);
}

// =============================================================================
// 10. ManagedMap
// =============================================================================

#[test]
fn managed_map_from_map_and_into_map() {
    let mut hm = HashMap::new();
    hm.insert("k1", 1);
    hm.insert("k2", 2);
    let map = ManagedMap::from_map(hm);
    assert_eq!(map.len(), 2);
    let back = map.into_map();
    assert_eq!(back.len(), 2);
}

#[test]
fn managed_map_insert_get_contains_remove() {
    let mut map = ManagedMap::<&str, i32>::new();
    assert!(map.is_empty());
    assert!(map.insert("a", 1).is_none()); // 新增返回 None
    assert_eq!(map.insert("a", 2), Some(1)); // 覆盖返回旧值
    map.insert("b", 3);

    assert!(map.contains_key(&"a"));
    assert_eq!(map.get(&"a"), Some(&2));
    assert_eq!(map.get(&"missing"), None);
    assert_eq!(map.len(), 2);

    assert_eq!(map.remove(&"a"), Some(2));
    assert!(map.remove(&"a").is_none());
    assert_eq!(map.len(), 1);
}

#[test]
fn managed_map_source_and_type_names() {
    let map = ManagedMap::<String, i64>::new()
        .with_source_name("app.xml")
        .with_key_type_name("String")
        .with_value_type_name("i64");
    assert_eq!(map.source_name(), Some("app.xml"));
    assert_eq!(map.key_type_name(), Some("String"));
    assert_eq!(map.value_type_name(), Some("i64"));
}

#[test]
fn managed_map_conversions() {
    let mut hm = HashMap::new();
    hm.insert(1u8, 2u8);
    let managed: ManagedMap<u8, u8> = hm.into();
    let back: HashMap<u8, u8> = managed.into();
    assert_eq!(back.get(&1), Some(&2));
}

// =============================================================================
// 11. ManagedProperties
// =============================================================================

#[test]
fn managed_properties_set_get_contains() {
    let mut props = ManagedProperties::new();
    assert!(props.is_empty());
    props.set_property("host", "localhost");
    props.set_property("port", "8080");

    assert_eq!(props.get_property("host"), Some("localhost"));
    assert!(props.contains_property("port"));
    assert!(!props.contains_property("missing"));
    assert_eq!(props.len(), 2);
}

#[test]
fn managed_properties_remove_and_names() {
    let mut props = ManagedProperties::new();
    props.set_property("a", "1");
    props.set_property("b", "2");

    assert_eq!(props.remove_property("a"), Some("1".to_string()));
    assert!(props.remove_property("a").is_none());
    assert_eq!(props.len(), 1);

    let names = props.property_names();
    assert_eq!(names.len(), 1);
    assert!(names.contains(&"b"));
}

#[test]
fn managed_properties_merge_flag() {
    let mut props = ManagedProperties::new();
    assert!(!props.is_merge_enabled());
    props.set_merge_enabled(true);
    assert!(props.is_merge_enabled());
    props.set_merge_enabled(false);
    assert!(!props.is_merge_enabled());
}

#[test]
fn managed_properties_from_map_and_source() {
    let mut hm = HashMap::new();
    hm.insert("x".to_string(), "1".to_string());
    let props = ManagedProperties::from_map(hm).with_source_name("props.xml");
    assert_eq!(props.source_name(), Some("props.xml"));
    assert_eq!(props.get_property("x"), Some("1"));
    let back: HashMap<String, String> = props.into();
    assert_eq!(back.get("x"), Some(&"1".to_string()));
}

// =============================================================================
// 12. ManagedArray
// =============================================================================

#[test]
fn managed_array_new_and_element_type_name() {
    let arr = ManagedArray::<i32>::new("int");
    assert_eq!(arr.element_type_name(), "int");
    assert!(arr.is_empty());
    assert_eq!(arr.len(), 0);
    assert!(arr.source_name().is_none());
}

#[test]
fn managed_array_push_and_len() {
    let mut arr = ManagedArray::<&str>::new("String");
    arr.push("a");
    arr.push("b");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr.as_slice(), &["a", "b"]);
}

#[test]
fn managed_array_from_vec_and_source() {
    let arr = ManagedArray::from_vec(vec![1u32, 2], "u32").with_source_name("arr.xml");
    assert_eq!(arr.element_type_name(), "u32");
    assert_eq!(arr.source_name(), Some("arr.xml"));
    assert_eq!(arr.into_vec(), vec![1u32, 2]);
}

// =============================================================================
// 13. MethodOverride / MethodOverrideBase / GenericMethodOverride
// =============================================================================

#[test]
fn method_override_base_builders_and_accessors() {
    let base = MethodOverrideBase::new("doWork")
        .with_overloaded(true)
        .with_source_name("src.xml");
    assert_eq!(base.method_name(), "doWork");
    assert!(base.overloaded());
    assert_eq!(base.source_name(), Some("src.xml"));
}

#[test]
fn method_override_base_defaults() {
    let base = MethodOverrideBase::new("m");
    assert!(!base.overloaded());
    assert!(base.source_name().is_none());
}

#[test]
fn generic_method_override_trait_dispatch() {
    let ov = GenericMethodOverride::new("create").with_overloaded(true);
    let dyn_ov: &dyn MethodOverride = &ov;
    assert_eq!(dyn_ov.get_method_name(), "create");
    assert!(dyn_ov.is_overloaded());
    assert!(dyn_ov.matches()); // 默认 true
    assert!(dyn_ov.validate().is_ok());
}

#[test]
fn generic_method_override_empty_name_validate_errors() {
    let ov = GenericMethodOverride::new("");
    assert!(ov.validate().is_err());
}

// =============================================================================
// 14. MethodOverrides
// =============================================================================

#[test]
fn method_overrides_add_multiple_and_size() {
    let mut mo = MethodOverrides::new();
    assert!(mo.is_empty());
    mo.add_override(GenericMethodOverride::new("a"));
    mo.add_override(GenericMethodOverride::new("b"));
    mo.add_override(GenericMethodOverride::new("c"));
    assert_eq!(mo.size(), 3);
    assert!(!mo.is_empty());
    assert_eq!(mo.get_overrides().len(), 3);
}

#[test]
fn method_overrides_add_same_name_replaces() {
    let mut mo = MethodOverrides::new();
    mo.add_override(GenericMethodOverride::new("dup"));
    mo.add_override(GenericMethodOverride::new("other"));
    // 同名覆盖应替换旧的，总数不变
    mo.add_override(GenericMethodOverride::new("dup"));
    assert_eq!(mo.size(), 2);
}

#[test]
fn method_overrides_get_and_contains_by_method_name() {
    let mut mo = MethodOverrides::new();
    mo.add_override(GenericMethodOverride::new("find"));
    assert!(mo.contains("find"));
    assert!(!mo.contains("absent"));
    let got = mo.get_override("find").expect("present");
    assert_eq!(got.get_method_name(), "find");
    assert!(mo.get_override("absent").is_none());
}

#[test]
fn method_overrides_remove_by_method_name_and_clear() {
    let mut mo = MethodOverrides::new();
    mo.add_override(GenericMethodOverride::new("a"));
    mo.add_override(GenericMethodOverride::new("b"));

    assert!(mo.remove("a"));
    assert!(!mo.remove("a")); // 再删返回 false
    assert_eq!(mo.size(), 1);

    mo.clear();
    assert!(mo.is_empty());
    assert_eq!(mo.size(), 0);
}

#[test]
fn method_overrides_add_with_arc() {
    let mut mo = MethodOverrides::new();
    let arc: Arc<dyn MethodOverride> = Arc::new(GenericMethodOverride::new("z"));
    mo.add(arc);
    assert_eq!(mo.size(), 1);
    assert!(mo.contains("z"));
}

// =============================================================================
// 15. LookupOverride
// =============================================================================

#[test]
fn lookup_override_new_and_accessors() {
    let lo = LookupOverride::new("createCommand", "command").with_type_name("Command");
    assert_eq!(lo.get_method_name(), "createCommand");
    assert_eq!(lo.get_bean_name(), "command");
    assert_eq!(lo.get_type_name(), Some("Command"));
}

#[test]
fn lookup_override_trait_impl() {
    let lo = LookupOverride::new("m", "bean");
    let dyn_ov: &dyn MethodOverride = &lo;
    assert_eq!(dyn_ov.get_method_name(), "m");
    assert!(!dyn_ov.is_overloaded());
    assert!(dyn_ov.validate().is_ok());
}

#[test]
fn lookup_override_validate_errors() {
    // 空方法名 → 错误
    let lo = LookupOverride::new("", "bean");
    assert!(lo.validate().is_err());

    // 方法名有效但既无 bean 名也无类型名 → 错误
    let lo2 = LookupOverride::new("m", "");
    assert!(lo2.validate().is_err());

    // 仅有类型名（无 bean 名）应合法
    let lo3 = LookupOverride::new("m", "").with_type_name("T");
    assert!(lo3.validate().is_ok());
}

// =============================================================================
// 16. ReplaceOverride
// =============================================================================

#[test]
fn replace_override_new_and_accessors() {
    let mut ro = ReplaceOverride::new("compute", "replacerBean");
    ro.add_type_identifier("int");
    ro.add_type_identifier("String");
    let ro = ro.with_overloaded(true);

    assert_eq!(ro.get_method_name(), "compute");
    assert_eq!(ro.get_method_replacer(), "replacerBean");
    assert_eq!(
        ro.type_identifiers(),
        &["int".to_string(), "String".to_string()]
    );
}

#[test]
fn replace_override_trait_impl() {
    let ro = ReplaceOverride::new("compute", "replacerBean");
    let dyn_ov: &dyn MethodOverride = &ro;
    assert_eq!(dyn_ov.get_method_name(), "compute");
    assert!(!dyn_ov.is_overloaded());
    assert!(dyn_ov.validate().is_ok());
}

#[test]
fn replace_override_validate_errors() {
    // 空方法名
    let ro = ReplaceOverride::new("", "replacer");
    assert!(ro.validate().is_err());
    // 空 replacer 名
    let ro2 = ReplaceOverride::new("m", "");
    assert!(ro2.validate().is_err());
}

// =============================================================================
// 17. MethodReplacer
// =============================================================================

#[test]
fn method_replacer_trait_reimplement() {
    /// 一个简单的替换器：忽略目标，返回固定值。
    struct FixedReplacer;
    impl MethodReplacer for FixedReplacer {
        fn reimplement(
            &self,
            _obj: &dyn Any,
            method: &str,
            _args: &[&dyn Any],
        ) -> Result<Box<dyn Any>, Box<dyn std::error::Error + Send + Sync>> {
            if method.is_empty() {
                return Err("empty method".into());
            }
            Ok(Box::new(99i32))
        }
    }

    let replacer = FixedReplacer;
    let target = 0u32;
    let res = replacer
        .reimplement(&target as &dyn Any, "doStuff", &[])
        .unwrap();
    let down = res.downcast_ref::<i32>();
    assert_eq!(down, Some(&99));

    // 空方法名 → 错误路径
    let err = replacer.reimplement(&target as &dyn Any, "", &[]);
    assert!(err.is_err());
}

// =============================================================================
// 18. MethodDescriptor
// =============================================================================

#[test]
fn method_descriptor_new_and_getters() {
    let md = MethodDescriptor::new("execute", "com.svc.Runner");
    assert_eq!(md.name(), "execute");
    assert_eq!(md.declaring_class(), "com.svc.Runner");
    assert_eq!(md.return_type(), ""); // 默认空
    assert_eq!(md.parameter_count(), 0);
    assert!(md.is_no_arg());
    assert_eq!(md.parameter_types().len(), 0);
}

#[test]
fn method_descriptor_with_all_and_signature() {
    let md = MethodDescriptor::with_all(
        "execute",
        "Runner",
        "void",
        vec!["i32".to_string(), "String".to_string()],
    );
    assert_eq!(md.return_type(), "void");
    assert_eq!(md.parameter_count(), 2);
    assert!(!md.is_no_arg());
    assert_eq!(md.signature(), "Runner#execute(i32, String)");
}

#[test]
fn method_descriptor_set_return_and_add_param() {
    let mut md = MethodDescriptor::new("run", "C");
    md.set_return_type("bool");
    md.add_parameter_type("u32");
    assert_eq!(md.return_type(), "bool");
    assert_eq!(md.parameter_count(), 1);
    assert_eq!(md.parameter_types(), &["u32".to_string()]);
}

// =============================================================================
// 19. CustomEditorConfigurer
// =============================================================================

/// 用于测试的最简属性编辑器：在字符串与 i32 间转换。
struct IntEditor {
    value: Option<Arc<dyn Any + Send + Sync>>,
}

impl IntEditor {
    fn new() -> Self {
        Self { value: None }
    }
}

impl PropertyEditor for IntEditor {
    fn target_type(&self) -> TypeId {
        TypeId::of::<i32>()
    }
    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let n: i32 = text.parse().map_err(
            |e: std::num::ParseIntError| -> Box<dyn std::error::Error + Send + Sync> {
                e.to_string().into()
            },
        )?;
        self.value = Some(Arc::new(n));
        Ok(())
    }
    fn get_as_text(&self) -> Option<String> {
        self.value
            .as_ref()
            .and_then(|v| v.downcast_ref::<i32>())
            .map(|n| n.to_string())
    }
    fn set_value(&mut self, value: Arc<dyn Any + Send + Sync>) {
        self.value = Some(value);
    }
    fn get_value(&self) -> Option<&dyn Any> {
        self.value.as_deref().map(|v| v as &dyn Any)
    }
    fn get_value_type(&self) -> TypeId {
        TypeId::of::<i32>()
    }
}

#[test]
fn custom_editor_configurer_register_get_has_count() {
    let mut cfg = CustomEditorConfigurer::new();
    assert!(cfg.is_empty());
    assert_eq!(cfg.len(), 0);

    cfg.register_custom_editor(TypeId::of::<i32>(), Box::new(IntEditor::new()));
    assert_eq!(cfg.len(), 1);
    assert!(!cfg.is_empty());
    assert!(cfg.has_custom_editor(TypeId::of::<i32>()));
    assert!(!cfg.has_custom_editor(TypeId::of::<u32>()));

    let editor = cfg.get_custom_editor(TypeId::of::<i32>());
    assert!(editor.is_some());
    assert_eq!(editor.unwrap().target_type(), TypeId::of::<i32>());

    // get 缺失返回 None
    assert!(cfg.get_custom_editor(TypeId::of::<u32>()).is_none());
}

#[test]
fn custom_editor_configurer_registered_types() {
    let mut cfg = CustomEditorConfigurer::new();
    cfg.register_custom_editor(TypeId::of::<i32>(), Box::new(IntEditor::new()));
    let types = cfg.registered_types();
    assert_eq!(types, vec![TypeId::of::<i32>()]);
}

#[test]
fn custom_editor_configurer_remove_and_clear() {
    let mut cfg = CustomEditorConfigurer::new();
    cfg.register_custom_editor(TypeId::of::<i32>(), Box::new(IntEditor::new()));
    assert!(cfg.remove_custom_editor(TypeId::of::<i32>()));
    assert!(!cfg.remove_custom_editor(TypeId::of::<i32>())); // 再删返回 false
    assert_eq!(cfg.len(), 0);

    cfg.register_custom_editor(TypeId::of::<i32>(), Box::new(IntEditor::new()));
    cfg.clear();
    assert!(cfg.is_empty());
}

// =============================================================================
// 20. BeanWiringInfo
// =============================================================================

#[test]
fn bean_wiring_info_autowire_by_name() {
    let info = BeanWiringInfo::autowire_by_name();
    assert_eq!(info.wiring_mode(), "byName");
    assert!(info.indicates_autowiring());
    // byName 默认不注入默认依赖 → 依赖名应为模式名
    assert_eq!(info.get_dependency_name(), Some("byName"));
    assert!(!info.indicates_default_dependency());
    assert!(info.bean_name().is_none());
    assert!(!info.ignore_unresolved());
}

#[test]
fn bean_wiring_info_autowire_by_type_with_default_dependency() {
    let info = BeanWiringInfo::autowire_by_type(true);
    assert_eq!(info.wiring_mode(), "byType");
    assert!(info.indicates_autowiring());
    assert!(info.indicates_default_dependency());
    // 当 default_dependency 为 true 时，get_dependency_name 返回 None
    assert_eq!(info.get_dependency_name(), None);
}

#[test]
fn bean_wiring_info_autowire_by_type_without_default_dependency() {
    let info = BeanWiringInfo::autowire_by_type(false);
    assert!(info.indicates_autowiring());
    assert!(!info.indicates_default_dependency());
    assert_eq!(info.get_dependency_name(), Some("byType"));
}

#[test]
fn bean_wiring_info_explicit_name_constructor() {
    let info = BeanWiringInfo::new("myBean", false);
    assert_eq!(info.bean_name(), Some("myBean"));
    assert!(!info.indicates_autowiring()); // 非自动装配
    assert!(!info.indicates_default_dependency());
    // 非自动装配 → get_dependency_name 为 None
    assert_eq!(info.get_dependency_name(), None);
    assert_eq!(info.wiring_mode(), "");
}

#[test]
fn bean_wiring_info_ignore_unresolved_toggle() {
    let mut info = BeanWiringInfo::new("b", true);
    assert!(!info.ignore_unresolved());
    info.set_ignore_unresolved(true);
    assert!(info.ignore_unresolved());
}
