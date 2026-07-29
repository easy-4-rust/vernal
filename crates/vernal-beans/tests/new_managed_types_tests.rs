//! 综合覆盖测试：第二批新创建的 Spring 风格托管类型（20 个源文件）。
//!
//! 每个源文件覆盖 2-3 个聚焦测试，覆盖主要 API 方法。

use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use vernal_beans::{
    AutowiredFieldElement, AutowiredMethodElement, BeanDefinition, BeanDefinitionRegistry,
    BeanWiringInfo, CustomAutowireConfigurer, CustomEditorConfigurer, GenericMethodOverride,
    InjectedElement, InjectionMetadata, LookupOverride, ManagedArray, ManagedList, ManagedMap,
    ManagedProperties, ManagedSet, MethodDescriptor, MethodOverride, MethodOverrideBase,
    MethodOverrides, MethodReplacer, NoopCustomAutowireConfigurer, PropertyEditor, ReplaceOverride,
    RootBeanDefinition, SimpleBeanDefinitionRegistry, StaticListableBeanFactory,
};

// =============================================================================
// 1. InjectedElement
// =============================================================================

#[test]
fn injected_element_new_defaults() {
    let el = InjectedElement::new("service");
    assert_eq!(el.member_name(), "service");
    assert!(!el.is_required());
    assert!(!el.is_injected());
}

#[test]
fn injected_element_with_and_set_required() {
    let el = InjectedElement::new("dao").with_required(true);
    assert!(el.is_required());

    let mut other = InjectedElement::new("x");
    other.set_required(true);
    assert!(other.is_required());
    other.set_required(false);
    assert!(!other.is_required());
}

#[test]
fn injected_element_inject_marks_and_is_idempotent() {
    let mut el = InjectedElement::new("field");
    assert!(!el.is_injected());
    let mut target = 0i32;
    el.inject(&mut target as &mut dyn Any, None).unwrap();
    assert!(el.is_injected());
    // 重复注入直接返回 Ok，不报错。
    el.inject(&mut target as &mut dyn Any, None).unwrap();
    assert!(el.is_injected());
}

#[test]
fn injected_element_clear_injected_resets() {
    let mut el = InjectedElement::new("field");
    let mut target = 0u64;
    el.inject(&mut target as &mut dyn Any, None).unwrap();
    assert!(el.is_injected());
    el.clear_injected();
    assert!(!el.is_injected());
}

// =============================================================================
// 2. InjectionMetadata
// =============================================================================

#[test]
fn injection_metadata_new_is_empty() {
    let md = InjectionMetadata::new("com.example.Service");
    assert_eq!(md.target_class(), "com.example.Service");
    assert!(md.is_empty());
    assert_eq!(md.len(), 0);
}

#[test]
fn injection_metadata_add_injected_element_tracks_len() {
    let mut md = InjectionMetadata::new("Service");
    md.add_injected_element(InjectedElement::new("a"));
    md.add_injected_element(InjectedElement::new("b"));
    assert_eq!(md.len(), 2);
    assert!(!md.is_empty());
    assert_eq!(md.injected_elements().len(), 2);
    assert_eq!(md.injected_elements()[0].member_name(), "a");
}

#[test]
fn injection_metadata_inject_and_clear_roundtrip() {
    let mut md = InjectionMetadata::new("Service");
    md.add_injected_element(InjectedElement::new("f1"));
    md.add_injected_element(InjectedElement::new("f2"));

    let mut target = 0i32;
    md.inject(&mut target as &mut dyn Any, None).unwrap();
    for el in md.injected_elements() {
        assert!(el.is_injected());
    }

    md.clear();
    for el in md.injected_elements() {
        assert!(!el.is_injected());
    }
}

#[test]
fn injection_metadata_add_injection_point_stored() {
    use vernal_beans::InjectionPoint;
    let mut md = InjectionMetadata::new("Service");
    let pt = InjectionPoint::new(TypeId::of::<u32>(), "u32");
    md.add_injection_point(pt);
    assert_eq!(md.injection_points().len(), 1);
    assert_eq!(md.len(), 1);
}

// =============================================================================
// 3. AutowiredFieldElement
// =============================================================================

#[test]
fn autowired_field_element_new_records_metadata() {
    let el = AutowiredFieldElement::new::<u64>("id", true);
    assert_eq!(el.field_name(), "id");
    assert_eq!(el.field_type_id(), TypeId::of::<u64>());
    assert!(el.required());
    assert_eq!(el.base().member_name(), "id");
    assert!(el.base().is_required());
}

#[test]
fn autowired_field_element_resolve_required_missing_errors() {
    let el = AutowiredFieldElement::new::<i32>("counter", true);
    let err = el
        .resolve(|_| None)
        .expect_err("required field without bean must error");
    let msg = err.to_string();
    assert!(msg.contains("counter"), "unexpected error: {msg}");
}

#[test]
fn autowired_field_element_resolve_optional_missing_ok_none() {
    let el = AutowiredFieldElement::new::<i32>("counter", false);
    let resolved = el.resolve(|_| None).unwrap();
    assert!(resolved.is_none());
}

#[test]
fn autowired_field_element_resolve_returns_value() {
    let el = AutowiredFieldElement::new::<i32>("counter", true);
    let v: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    let got = el
        .resolve(|tid| {
            if tid == TypeId::of::<i32>() {
                Some(v.clone())
            } else {
                None
            }
        })
        .unwrap();
    assert!(got.is_some());
    let down = got.unwrap();
    let n = down.downcast_ref::<i32>();
    assert_eq!(n, Some(&42));
}

// =============================================================================
// 4. AutowiredMethodElement
// =============================================================================

#[test]
fn autowired_method_element_new_records_metadata() {
    let el = AutowiredMethodElement::new(
        "configure",
        vec![TypeId::of::<i32>(), TypeId::of::<String>()],
        true,
    );
    assert_eq!(el.method_name(), "configure");
    assert_eq!(el.parameter_count(), 2);
    assert_eq!(el.parameter_type_ids().len(), 2);
    assert!(el.required());
}

#[test]
fn autowired_method_element_invoke_collects_args() {
    let el = AutowiredMethodElement::new("configure", vec![TypeId::of::<i32>()], true);
    let v: Arc<dyn Any + Send + Sync> = Arc::new(7i32);
    let args = el
        .invoke(|tid| {
            if tid == TypeId::of::<i32>() {
                Some(v.clone())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].downcast_ref::<i32>(), Some(&7));
}

#[test]
fn autowired_method_element_invoke_required_missing_errors() {
    let el = AutowiredMethodElement::new("configure", vec![TypeId::of::<bool>()], true);
    let err = el.invoke(|_| None).expect_err("must error");
    let msg = err.to_string();
    assert!(msg.contains("configure"), "unexpected error: {msg}");
}

// =============================================================================
// 5. CustomAutowireConfigurer
// =============================================================================

#[test]
fn noop_custom_autowire_configurer_returns_false() {
    let configurer = NoopCustomAutowireConfigurer;
    let def = test_bean_definition("foo");
    assert!(!configurer.is_custom_autowire_candidate(&*def));
}

#[test]
fn custom_autowire_configurer_default_trait_method_is_false() {
    // 通过 trait 对象验证默认实现。
    let cfg: Box<dyn CustomAutowireConfigurer> = Box::new(NoopCustomAutowireConfigurer);
    let def = test_bean_definition("bar");
    assert!(!cfg.is_custom_autowire_candidate(&*def));
}

#[test]
fn custom_autowire_configurer_user_impl_can_return_true() {
    struct AlwaysCandidate;
    impl CustomAutowireConfigurer for AlwaysCandidate {
        fn is_custom_autowire_candidate(&self, _d: &dyn BeanDefinition) -> bool {
            true
        }
    }
    let cfg = AlwaysCandidate;
    let def = test_bean_definition("baz");
    assert!(cfg.is_custom_autowire_candidate(&*def));
}

/// 构造一个可用于测试的 BeanDefinition。
fn test_bean_definition(name: &str) -> Arc<dyn BeanDefinition> {
    use vernal_beans::{ComponentKey, Scope};
    #[derive(Debug)]
    struct StubDef {
        key: ComponentKey,
    }
    impl BeanDefinition for StubDef {
        fn bean_name(&self) -> &ComponentKey {
            &self.key
        }
        fn bean_class_name(&self) -> &str {
            "StubDef"
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
    let _ = name; // 名称仅为可读性
    Arc::new(StubDef {
        key: ComponentKey::of::<u64>(),
    })
}

// =============================================================================
// 6. SimpleBeanDefinitionRegistry
// =============================================================================

fn boxed_def() -> Box<dyn BeanDefinition> {
    // RootBeanDefinition 实现了 BeanDefinition trait。
    Box::new(RootBeanDefinition::new())
}

#[test]
fn simple_registry_register_get_contains() {
    let mut reg = SimpleBeanDefinitionRegistry::new();
    assert_eq!(reg.bean_definition_count(), 0);
    reg.register_bean_definition("a".to_string(), boxed_def())
        .unwrap();
    assert!(reg.contains_bean_definition("a"));
    assert!(!reg.contains_bean_definition("missing"));
    assert!(reg.get_bean_definition("a").is_some());
    assert!(reg.get_bean_definition("missing").is_none());
    assert_eq!(reg.bean_definition_count(), 1);
}

#[test]
fn simple_registry_empty_name_errors() {
    let mut reg = SimpleBeanDefinitionRegistry::new();
    let err = reg
        .register_bean_definition(String::new(), boxed_def())
        .expect_err("empty name must error");
    assert!(err.to_string().contains("empty"));
}

#[test]
fn simple_registry_remove_and_clear() {
    let mut reg = SimpleBeanDefinitionRegistry::new();
    reg.register_bean_definition("a".to_string(), boxed_def())
        .unwrap();
    reg.register_bean_definition("b".to_string(), boxed_def())
        .unwrap();

    let removed = reg.remove_bean_definition("a").unwrap();
    assert_eq!(removed.bean_class_name(), "unknown");
    assert!(!reg.contains_bean_definition("a"));
    assert_eq!(reg.bean_definition_count(), 1);

    // remove missing errors.
    reg.remove_bean_definition("a").expect_err("should error");

    reg.clear();
    assert_eq!(reg.bean_definition_count(), 0);
    assert!(reg.bean_definition_names().is_empty());
}

#[test]
fn simple_registry_names_and_override() {
    let mut reg = SimpleBeanDefinitionRegistry::new();
    reg.register_bean_definition("a".to_string(), boxed_def())
        .unwrap();
    reg.register_bean_definition("b".to_string(), boxed_def())
        .unwrap();
    let mut names = reg.bean_definition_names();
    names.sort();
    assert_eq!(names, vec!["a".to_string(), "b".to_string()]);

    // 覆盖语义：同名再注册覆盖旧定义。
    reg.register_bean_definition("a".to_string(), boxed_def())
        .unwrap();
    assert_eq!(reg.bean_definition_count(), 2);
}

// =============================================================================
// 7. StaticListableBeanFactory
// =============================================================================

#[test]
fn static_factory_register_and_get() {
    let mut f = StaticListableBeanFactory::new();
    assert!(f.is_empty());
    f.register_singleton("answer", 42i32);
    assert!(!f.is_empty());
    assert_eq!(f.bean_count(), 1);
    assert!(f.contains_bean("answer"));

    let bean = f.get_bean("answer").unwrap();
    assert_eq!(bean.downcast_ref::<i32>(), Some(&42));
}

#[test]
fn static_factory_get_typed_and_type_info() {
    let mut f = StaticListableBeanFactory::new();
    f.register_singleton("answer", 42i32);

    let typed = f.get_typed_bean::<i32>("answer").unwrap();
    assert_eq!(*typed.as_ref(), 42);
    assert_eq!(f.get_type_id("answer"), Some(TypeId::of::<i32>()));
    assert!(f.get_type("answer").is_some());

    // 类型不匹配应失败。
    f.get_typed_bean::<String>("answer")
        .expect_err("type mismatch must error");
}

#[test]
fn static_factory_missing_bean_errors() {
    let f = StaticListableBeanFactory::new();
    let err = f.get_bean("none").expect_err("missing bean must error");
    assert!(err.to_string().contains("none"));
}

#[test]
fn static_factory_names_for_type_and_remove() {
    let mut f = StaticListableBeanFactory::new();
    f.register_singleton("a", 1i32);
    f.register_singleton("b", 2i32);
    f.register_singleton("s", "hi");

    let mut i32_names = f.bean_names_for_type(TypeId::of::<i32>());
    i32_names.sort();
    assert_eq!(i32_names, vec!["a".to_string(), "b".to_string()]);

    assert!(f.remove_bean("a"));
    assert!(!f.contains_bean("a"));
    assert!(!f.remove_bean("a"));

    f.clear();
    assert!(f.is_empty());
}

// =============================================================================
// 8. ManagedList
// =============================================================================

#[test]
fn managed_list_new_push_len() {
    let mut list: ManagedList<i32> = ManagedList::new();
    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
    list.push(1);
    list.push(2);
    assert_eq!(list.len(), 2);
    assert!(!list.is_empty());
    assert_eq!(list.as_slice(), &[1, 2]);
}

#[test]
fn managed_list_from_and_into_vec() {
    let list = ManagedList::from_vec(vec![10, 20, 30]);
    assert_eq!(list.len(), 3);
    let vec: Vec<i32> = list.into_vec();
    assert_eq!(vec, vec![10, 20, 30]);

    // From/Into 转换。
    let list2: ManagedList<i32> = vec![1, 2].into();
    let back: Vec<i32> = list2.into();
    assert_eq!(back, vec![1, 2]);
}

#[test]
fn managed_list_source_and_element_type_name() {
    let list = ManagedList::from_vec(vec![1])
        .with_source_name("beans.xml")
        .with_element_type_name("java.lang.Integer");
    assert_eq!(list.source_name(), Some("beans.xml"));
    assert_eq!(list.element_type_name(), Some("java.lang.Integer"));
}

// =============================================================================
// 9. ManagedSet
// =============================================================================

#[test]
fn managed_set_new_insert_contains() {
    let mut set: ManagedSet<String> = ManagedSet::new();
    assert!(set.is_empty());
    assert!(set.insert("a".to_string()));
    assert!(!set.insert("a".to_string())); // 已存在返回 false。
    set.insert("b".to_string());
    assert_eq!(set.len(), 2);
    assert!(set.contains(&"a".to_string()));
    assert!(!set.contains(&"z".to_string()));
}

#[test]
fn managed_set_from_set_and_remove() {
    let mut inner = HashSet::new();
    inner.insert("x".to_string());
    inner.insert("y".to_string());
    let mut set = ManagedSet::from_set(inner);
    assert_eq!(set.len(), 2);
    assert!(set.remove(&"x".to_string()));
    assert!(!set.contains(&"x".to_string()));
    assert_eq!(set.len(), 1);

    // From/Into 转换。
    let managed: ManagedSet<i32> = {
        let mut s = HashSet::new();
        s.insert(1);
        s
    }
    .into();
    let back: HashSet<i32> = managed.into();
    assert_eq!(back.len(), 1);
}

#[test]
fn managed_set_source_name() {
    let set: ManagedSet<i32> = ManagedSet::new().with_source_name("cfg.xml");
    assert_eq!(set.source_name(), Some("cfg.xml"));
    assert_eq!(set.element_type_name(), None);
}

// =============================================================================
// 10. ManagedMap
// =============================================================================

#[test]
fn managed_map_new_insert_get() {
    let mut map: ManagedMap<String, i32> = ManagedMap::new();
    assert!(map.is_empty());
    assert_eq!(map.insert("a".to_string(), 1), None);
    assert_eq!(map.insert("a".to_string(), 2), Some(1)); // 返回旧值。
    assert_eq!(map.len(), 1);
    assert_eq!(map.get(&"a".to_string()), Some(&2));
    assert!(map.contains_key(&"a".to_string()));
    assert!(!map.contains_key(&"z".to_string()));
}

#[test]
fn managed_map_from_map_and_remove() {
    let mut inner = HashMap::new();
    inner.insert("k".to_string(), 9);
    let mut map = ManagedMap::from_map(inner);
    assert_eq!(map.len(), 1);
    assert_eq!(map.remove(&"k".to_string()), Some(9));
    assert!(map.is_empty());

    // From/Into 转换。
    let managed: ManagedMap<String, i32> = HashMap::new().into();
    let _back: HashMap<String, i32> = managed.into();
}

#[test]
fn managed_map_key_value_type_names() {
    let map: ManagedMap<String, i32> = ManagedMap::new()
        .with_key_type_name("String")
        .with_value_type_name("Integer")
        .with_source_name("s");
    assert_eq!(map.key_type_name(), Some("String"));
    assert_eq!(map.value_type_name(), Some("Integer"));
    assert_eq!(map.source_name(), Some("s"));
}

// =============================================================================
// 11. ManagedProperties
// =============================================================================

#[test]
fn managed_properties_set_get_remove() {
    let mut props = ManagedProperties::new();
    assert!(props.is_empty());
    props.set_property("a", "1");
    props.set_property("b", "2");
    assert_eq!(props.len(), 2);
    assert_eq!(props.get_property("a"), Some("1"));
    assert!(props.contains_property("b"));
    assert_eq!(props.remove_property("a"), Some("1".to_string()));
    assert!(!props.contains_property("a"));
    assert!(props.get_property("missing").is_none());
}

#[test]
fn managed_properties_property_names() {
    let mut props = ManagedProperties::new();
    props.set_property("x", "10");
    props.set_property("y", "20");
    let mut names = props.property_names().into_iter().collect::<Vec<_>>();
    names.sort();
    assert_eq!(names, vec!["x", "y"]);
}

#[test]
fn managed_properties_merge_flag_and_source() {
    let mut props = ManagedProperties::new().with_source_name("p.xml");
    assert_eq!(props.source_name(), Some("p.xml"));
    assert!(!props.is_merge_enabled());
    props.set_merge_enabled(true);
    assert!(props.is_merge_enabled());
}

// =============================================================================
// 12. ManagedArray
// =============================================================================

#[test]
fn managed_array_new_and_element_type_name() {
    let arr: ManagedArray<i32> = ManagedArray::new("java.lang.Integer");
    assert_eq!(arr.element_type_name(), "java.lang.Integer");
    assert!(arr.is_empty());
}

#[test]
fn managed_array_push_and_from_vec() {
    let mut arr: ManagedArray<String> = ManagedArray::new("String");
    arr.push("a".to_string());
    arr.push("b".to_string());
    assert_eq!(arr.len(), 2);
    assert_eq!(arr.as_slice(), &["a".to_string(), "b".to_string()]);

    let from_vec = ManagedArray::from_vec(vec![1, 2, 3], "Integer");
    assert_eq!(from_vec.len(), 3);
    assert_eq!(from_vec.into_vec(), vec![1, 2, 3]);
}

#[test]
fn managed_array_source_name() {
    let arr: ManagedArray<i32> = ManagedArray::new("Integer").with_source_name("arr.xml");
    assert_eq!(arr.source_name(), Some("arr.xml"));
}

// =============================================================================
// 13. MethodOverride trait + MethodOverrideBase + GenericMethodOverride
// =============================================================================

#[test]
fn method_override_base_builder_methods() {
    let base = MethodOverrideBase::new("create")
        .with_overloaded(true)
        .with_source_name("over.xml");
    assert_eq!(base.method_name(), "create");
    assert!(base.overloaded());
    assert_eq!(base.source_name(), Some("over.xml"));
}

#[test]
fn generic_method_override_implements_trait() {
    let mo = GenericMethodOverride::new("doStuff").with_overloaded(true);
    assert_eq!(mo.get_method_name(), "doStuff");
    assert!(mo.is_overloaded());
    assert!(mo.matches()); // 默认 true。
    mo.validate().unwrap();
}

#[test]
fn generic_method_override_empty_name_validate_errors() {
    let mo = GenericMethodOverride::new("");
    let err = mo.validate().expect_err("empty name must error");
    assert!(err.to_string().contains("empty"));
}

// =============================================================================
// 14. MethodOverrides
// =============================================================================

#[test]
fn method_overrides_add_and_query() {
    let mut mos = MethodOverrides::new();
    assert!(mos.is_empty());
    assert_eq!(mos.size(), 0);

    mos.add_override(GenericMethodOverride::new("m1"));
    mos.add_override(GenericMethodOverride::new("m2"));
    assert_eq!(mos.size(), 2);
    assert!(mos.contains("m1"));
    assert!(mos.get_override("m2").is_some());
    assert!(!mos.is_empty());
}

#[test]
fn method_overrides_same_name_replaces() {
    let mut mos = MethodOverrides::new();
    mos.add_override(GenericMethodOverride::new("dup"));
    mos.add_override(GenericMethodOverride::new("dup").with_overloaded(true));
    // 同名替换，数量仍为 1。
    assert_eq!(mos.size(), 1);
    let o = mos.get_override("dup").unwrap();
    assert!(o.is_overloaded());
}

#[test]
fn method_overrides_remove_and_clear() {
    let mut mos = MethodOverrides::new();
    mos.add_override(GenericMethodOverride::new("a"));
    mos.add_override(GenericMethodOverride::new("b"));
    assert!(mos.remove("a"));
    assert!(!mos.contains("a"));
    assert_eq!(mos.size(), 1);
    assert!(!mos.remove("a"));

    mos.clear();
    assert!(mos.is_empty());
}

#[test]
fn method_overrides_get_overrides_slice() {
    let mut mos = MethodOverrides::new();
    mos.add_override(GenericMethodOverride::new("a"));
    let slice = mos.get_overrides();
    assert_eq!(slice.len(), 1);
    assert_eq!(slice[0].get_method_name(), "a");
}

// =============================================================================
// 15. LookupOverride
// =============================================================================

#[test]
fn lookup_override_new_accessors() {
    let lo = LookupOverride::new("createCommand", "commandPrototype");
    assert_eq!(lo.get_method_name(), "createCommand");
    assert_eq!(lo.get_bean_name(), "commandPrototype");
    assert!(lo.get_type_name().is_none());
}

#[test]
fn lookup_override_with_type_name() {
    let lo = LookupOverride::new("createCommand", "bean").with_type_name("Command");
    assert_eq!(lo.get_type_name(), Some("Command"));
}

#[test]
fn lookup_override_validate_errors() {
    // 方法名为空。
    let lo_empty = LookupOverride::new("", "b");
    let err = lo_empty
        .validate()
        .expect_err("empty method name must error");
    assert!(err.to_string().contains("method name"));

    // 既无 bean 名也无类型名。
    let lo_no_target = LookupOverride::new("m", "");
    let err = lo_no_target
        .validate()
        .expect_err("missing target must error");
    assert!(err.to_string().contains("bean name"));
}

#[test]
fn lookup_override_validate_ok() {
    let lo = LookupOverride::new("createCommand", "commandPrototype");
    lo.validate().unwrap();
}

// =============================================================================
// 16. ReplaceOverride
// =============================================================================

#[test]
fn replace_override_new_accessors() {
    let ro = ReplaceOverride::new("compute", "replacerBean");
    assert_eq!(ro.get_method_name(), "compute");
    assert_eq!(ro.get_method_replacer(), "replacerBean");
    assert!(ro.type_identifiers().is_empty());
}

#[test]
fn replace_override_add_type_identifier_and_overloaded() {
    let mut ro = ReplaceOverride::new("compute", "replacerBean");
    ro.add_type_identifier("java.lang.String");
    ro.add_type_identifier("int");
    assert_eq!(ro.type_identifiers().len(), 2);

    let ro2 = ReplaceOverride::new("compute", "replacerBean").with_overloaded(true);
    assert!(ro2.is_overloaded());
}

#[test]
fn replace_override_validate_errors() {
    let ro_empty_method = ReplaceOverride::new("", "replacer");
    ro_empty_method
        .validate()
        .expect_err("empty method name must error");

    let ro_empty_replacer = ReplaceOverride::new("m", "");
    ro_empty_replacer
        .validate()
        .expect_err("empty replacer must error");
}

#[test]
fn replace_override_validate_ok() {
    let ro = ReplaceOverride::new("compute", "replacerBean");
    ro.validate().unwrap();
}

// =============================================================================
// 17. MethodReplacer trait
// =============================================================================

#[test]
fn method_replacer_trait_object_dispatch() {
    struct DoubleReplacer;
    impl MethodReplacer for DoubleReplacer {
        fn reimplement(
            &self,
            _obj: &dyn Any,
            _method: &str,
            args: &[&dyn Any],
        ) -> Result<Box<dyn Any>, Box<dyn std::error::Error + Send + Sync>> {
            // 简单语义：取第一个 i32 参数并翻倍。
            let n = args
                .get(0)
                .and_then(|a| a.downcast_ref::<i32>())
                .copied()
                .unwrap_or(0);
            Ok(Box::new(n * 2))
        }
    }

    let replacer: Box<dyn MethodReplacer> = Box::new(DoubleReplacer);
    let arg = 21i32;
    let out = replacer
        .reimplement(&0i32 as &dyn Any, "compute", &[&arg as &dyn Any])
        .unwrap();
    assert_eq!(out.downcast_ref::<i32>(), Some(&42));
}

#[test]
fn method_replacer_can_return_error() {
    struct FailingReplacer;
    impl MethodReplacer for FailingReplacer {
        fn reimplement(
            &self,
            _obj: &dyn Any,
            _method: &str,
            _args: &[&dyn Any],
        ) -> Result<Box<dyn Any>, Box<dyn std::error::Error + Send + Sync>> {
            Err("not implemented".into())
        }
    }
    let replacer: Box<dyn MethodReplacer> = Box::new(FailingReplacer);
    let res = replacer.reimplement(&0i32 as &dyn Any, "m", &[]);
    assert!(res.is_err());
}

// =============================================================================
// 18. MethodDescriptor
// =============================================================================

#[test]
fn method_descriptor_new_and_accessors() {
    let md = MethodDescriptor::new("toString", "MyClass");
    assert_eq!(md.name(), "toString");
    assert_eq!(md.declaring_class(), "MyClass");
    assert_eq!(md.return_type(), "");
    assert!(md.is_no_arg());
    assert_eq!(md.parameter_count(), 0);
}

#[test]
fn method_descriptor_with_all_and_signature() {
    let md = MethodDescriptor::with_all(
        "setName",
        "User",
        "void",
        vec!["String".to_string(), "int".to_string()],
    );
    assert_eq!(md.return_type(), "void");
    assert_eq!(md.parameter_types().len(), 2);
    assert!(!md.is_no_arg());
    assert_eq!(md.signature(), "User#setName(String, int)");
}

#[test]
fn method_descriptor_mutators() {
    let mut md = MethodDescriptor::new("doWork", "Svc");
    md.set_return_type("bool");
    md.add_parameter_type("i32");
    assert_eq!(md.return_type(), "bool");
    assert_eq!(md.parameter_count(), 1);
}

// =============================================================================
// 19. CustomEditorConfigurer
// =============================================================================

/// 测试用的属性编辑器：解析 i32。
#[derive(Default)]
struct I32Editor {
    value: Option<Arc<dyn Any + Send + Sync>>,
}

impl PropertyEditor for I32Editor {
    fn target_type(&self) -> TypeId {
        TypeId::of::<i32>()
    }
    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let n: i32 = text.parse()?;
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
        self.value.as_ref().map(|v| v.as_ref() as &dyn Any)
    }
    fn get_value_type(&self) -> TypeId {
        TypeId::of::<i32>()
    }
}

#[test]
fn custom_editor_configurer_register_get_has() {
    let mut cfg = CustomEditorConfigurer::new();
    assert!(cfg.is_empty());
    assert_eq!(cfg.len(), 0);
    cfg.register_custom_editor(TypeId::of::<i32>(), Box::<I32Editor>::default());
    assert_eq!(cfg.len(), 1);
    assert!(cfg.has_custom_editor(TypeId::of::<i32>()));
    assert!(cfg.get_custom_editor(TypeId::of::<i32>()).is_some());
    assert!(!cfg.has_custom_editor(TypeId::of::<String>()));
}

#[test]
fn custom_editor_configurer_registered_types_and_remove_clear() {
    let mut cfg = CustomEditorConfigurer::new();
    cfg.register_custom_editor(TypeId::of::<i32>(), Box::<I32Editor>::default());
    let types = cfg.registered_types();
    assert_eq!(types, vec![TypeId::of::<i32>()]);

    assert!(cfg.remove_custom_editor(TypeId::of::<i32>()));
    assert!(!cfg.has_custom_editor(TypeId::of::<i32>()));
    assert!(!cfg.remove_custom_editor(TypeId::of::<i32>()));

    cfg.register_custom_editor(TypeId::of::<i32>(), Box::<I32Editor>::default());
    cfg.clear();
    assert!(cfg.is_empty());
}

#[test]
fn custom_editor_configurer_editor_round_trip() {
    let mut cfg = CustomEditorConfigurer::new();
    cfg.register_custom_editor(TypeId::of::<i32>(), Box::<I32Editor>::default());
    let editor = cfg.get_custom_editor(TypeId::of::<i32>()).unwrap();
    // PropertyEditor 的 target_type 一致。
    assert_eq!(editor.target_type(), TypeId::of::<i32>());
}

// =============================================================================
// 20. BeanWiringInfo
// =============================================================================

#[test]
fn bean_wiring_info_by_name_autowiring() {
    let info = BeanWiringInfo::autowire_by_name();
    assert!(info.indicates_autowiring());
    assert_eq!(info.wiring_mode(), "byName");
    assert_eq!(info.get_dependency_name(), Some("byName"));
    assert!(info.bean_name().is_none());
    assert!(!info.indicates_default_dependency());
}

#[test]
fn bean_wiring_info_by_type_no_default_has_dependency_name() {
    let info = BeanWiringInfo::autowire_by_type(false);
    assert!(info.indicates_autowiring());
    assert_eq!(info.wiring_mode(), "byType");
    assert_eq!(info.get_dependency_name(), Some("byType"));
}

#[test]
fn bean_wiring_info_by_type_with_default_no_dependency_name() {
    let info = BeanWiringInfo::autowire_by_type(true);
    assert!(info.indicates_autowiring());
    assert!(info.indicates_default_dependency());
    // 默认依赖时不返回依赖名。
    assert!(info.get_dependency_name().is_none());
}

#[test]
fn bean_wiring_info_named_not_autowiring() {
    let mut info = BeanWiringInfo::new("specificBean", false);
    assert!(!info.indicates_autowiring());
    assert_eq!(info.bean_name(), Some("specificBean"));
    assert!(info.get_dependency_name().is_none());
    assert!(!info.ignore_unresolved());
    info.set_ignore_unresolved(true);
    assert!(info.ignore_unresolved());
}
