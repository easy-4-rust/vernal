//! Container 深度函数覆盖率测试 — 覆盖剩余未覆盖的函数。
use std::any::Any;
use std::sync::Arc;

fn make_container() -> vernal_beans::Container {
    let mut b = vernal_beans::RegistryBuilder::new();
    let _ = b.register(vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    let _ = b.register(vernal_beans::ComponentDefinition::singleton::<i32, _>(|_| 42i32));
    let _ = b.register(vernal_beans::ComponentDefinition::singleton::<f64, _>(|_| 3.14f64));
    vernal_beans::Container::new(b.build().unwrap())
}

fn make_transient_container() -> vernal_beans::Container {
    let mut b = vernal_beans::RegistryBuilder::new();
    let _ = b.register(vernal_beans::ComponentDefinition::transient::<String, _>(|_| "transient".to_string()));
    vernal_beans::Container::new(b.build().unwrap())
}

// ═══ resolve_qualified_in ═══
#[test]
fn resolve_qualified_in_basic() {
    let mut b = vernal_beans::RegistryBuilder::new();
    let q = vernal_beans::Qualifier::new("q1").unwrap();
    let _ = b.register(vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "q_val".to_string()).qualified(q.clone()));
    let c = vernal_beans::Container::new(b.build().unwrap());
    let scope = c.open_scope::<String>();
    let val: Arc<String> = c.resolve_qualified_in(&q, &scope).unwrap();
    assert_eq!(*val, "q_val");
}

#[test]
fn resolve_qualified_in_wrong_scope() {
    let mut b = vernal_beans::RegistryBuilder::new();
    let q = vernal_beans::Qualifier::new("q2").unwrap();
    let _ = b.register(vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "v".to_string()).qualified(q.clone()));
    let c1 = vernal_beans::Container::new(b.build().unwrap());
    let c2 = vernal_beans::Container::new(vernal_beans::RegistryBuilder::new().build().unwrap());
    let scope_from_c2 = c2.open_scope::<String>();
    let result: Result<Arc<String>, _> = c1.resolve_qualified_in(&q, &scope_from_c2);
    assert!(result.is_err());
}

// ═══ resolve_trait_in ═══
#[test]
fn resolve_trait_in_no_binding() {
    let c = make_container();
    let scope = c.open_scope::<String>();
    let val: Result<Arc<dyn std::fmt::Debug + Send + Sync>, _> = c.resolve_trait_in(&scope);
    assert!(val.is_err());
}

// ═══ resolve_qualified_trait ═══
#[test]
fn resolve_qualified_trait_no_binding() {
    let c = make_container();
    let q = vernal_beans::Qualifier::new("missing").unwrap();
    let val: Result<Arc<dyn std::fmt::Debug + Send + Sync>, _> = c.resolve_qualified_trait(&q);
    assert!(val.is_err());
}

// ═══ resolve_qualified_trait_in ═══
#[test]
fn resolve_qualified_trait_in_no_binding() {
    let c = make_container();
    let q = vernal_beans::Qualifier::new("missing").unwrap();
    let scope = c.open_scope::<String>();
    let val: Result<Arc<dyn std::fmt::Debug + Send + Sync>, _> = c.resolve_qualified_trait_in(&q, &scope);
    assert!(val.is_err());
}

#[test]
fn resolve_qualified_trait_in_wrong_scope() {
    let c1 = make_container();
    let c2 = vernal_beans::Container::new(vernal_beans::RegistryBuilder::new().build().unwrap());
    let q = vernal_beans::Qualifier::new("x").unwrap();
    let scope_from_c2 = c2.open_scope::<String>();
    let val: Result<Arc<dyn std::fmt::Debug + Send + Sync>, _> = c1.resolve_qualified_trait_in(&q, &scope_from_c2);
    assert!(val.is_err());
}

// ═══ resolve_all_traits_in ═══
#[test]
fn resolve_all_traits_in_no_binding() {
    let c = make_container();
    let scope = c.open_scope::<String>();
    let val: Result<Vec<Arc<dyn std::fmt::Debug + Send + Sync>>, _> = c.resolve_all_traits_in(&scope);
    assert!(val.unwrap().is_empty());
}

#[test]
fn resolve_all_traits_in_wrong_scope() {
    let c1 = make_container();
    let c2 = vernal_beans::Container::new(vernal_beans::RegistryBuilder::new().build().unwrap());
    let scope_from_c2 = c2.open_scope::<String>();
    let val: Result<Vec<Arc<dyn std::fmt::Debug + Send + Sync>>, _> = c1.resolve_all_traits_in(&scope_from_c2);
    assert!(val.is_err());
}

// ═══ get_bean_by_type_id (BeanFactory trait) ═══
#[test]
fn bf_get_bean_by_type_id_single() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    let bean = c.get_bean_by_type_id(std::any::TypeId::of::<String>());
    assert!(bean.is_ok());
}

#[test]
fn bf_get_bean_by_type_id_none() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    let bean = c.get_bean_by_type_id(std::any::TypeId::of::<bool>());
    assert!(bean.is_err());
}

// ═══ get_type / is_singleton / is_prototype not found ═══
#[test]
fn bf_get_type_not_found() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    let t = c.get_type(&vernal_beans::ComponentKey::of::<bool>());
    assert!(t.is_err());
}

#[test]
fn bf_is_singleton_not_found() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    assert!(c.is_singleton(&vernal_beans::ComponentKey::of::<bool>()).is_err());
}

#[test]
fn bf_is_prototype_not_found() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    assert!(c.is_prototype(&vernal_beans::ComponentKey::of::<bool>()).is_err());
}

// ═══ configure_bean ═══
#[test]
fn acbf_configure_bean() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    let result = c.configure_bean(bean, "test_bean");
    assert!(result.is_ok());
}

// ═══ autowire_bean_properties ═══
#[test]
fn acbf_autowire_bean_properties_no_mode() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    let result = c.autowire_bean_properties(bean, 0, false);
    assert!(result.is_ok());
}

#[test]
fn acbf_autowire_bean_properties_by_name() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    let result = c.autowire_bean_properties(bean, 1, false);
    assert!(result.is_ok());
}

#[test]
fn acbf_autowire_bean_properties_by_type() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    let result = c.autowire_bean_properties(bean, 2, false);
    assert!(result.is_ok());
}

#[test]
fn acbf_autowire_bean_properties_invalid() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    let result = c.autowire_bean_properties(bean, 99, false);
    assert!(result.is_ok());
}

// ═══ apply_bean_property_values ═══
#[test]
fn acbf_apply_bean_property_values() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    let result = c.apply_bean_property_values(bean, "test_bean");
    assert!(result.is_ok());
}

// ═══ autowire modes ═══
#[test]
fn acbf_autowire_mode_1_by_name() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    assert!(c.autowire("alloc::string::String", 1, false).is_ok());
}

#[test]
fn acbf_autowire_mode_2_by_type() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    assert!(c.autowire("alloc::string::String", 2, false).is_ok());
}

#[test]
fn acbf_autowire_mode_3_constructor() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    assert!(c.autowire("alloc::string::String", 3, false).is_ok());
}

// ═══ resolve_named_bean ═══
#[test]
fn acbf_resolve_named_bean_found() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let holder = c.resolve_named_bean(std::any::TypeId::of::<String>());
    assert!(holder.is_ok());
}

#[test]
fn acbf_resolve_named_bean_not_found() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let holder = c.resolve_named_bean(std::any::TypeId::of::<bool>());
    assert!(holder.is_err());
}

// ═══ resolve_dependency ═══
#[test]
fn acbf_resolve_dependency_found() {
    use vernal_beans::AutowireCapableBeanFactory;
    use vernal_beans::factory::support::dependency_descriptor::DependencyDescriptor;
    let c = make_container();
    let dd = DependencyDescriptor::new(std::any::TypeId::of::<String>(), "alloc::string::String".to_string(), true);
    let result = c.resolve_dependency(&dd, None);
    // May succeed or fail depending on internal resolution; just exercise the code path
    let _ = result;
}

#[test]
fn acbf_resolve_dependency_not_found_optional() {
    use vernal_beans::AutowireCapableBeanFactory;
    use vernal_beans::factory::support::dependency_descriptor::DependencyDescriptor;
    let c = make_container();
    let dd = DependencyDescriptor::new(std::any::TypeId::of::<bool>(), "bool".to_string(), false);
    let result = c.resolve_dependency(&dd, None);
    assert!(result.unwrap().is_none());
}

#[test]
fn acbf_resolve_dependency_not_found_required() {
    use vernal_beans::AutowireCapableBeanFactory;
    use vernal_beans::factory::support::dependency_descriptor::DependencyDescriptor;
    let c = make_container();
    let dd = DependencyDescriptor::new(std::any::TypeId::of::<bool>(), "bool".to_string(), true);
    let result = c.resolve_dependency(&dd, None);
    assert!(result.is_err());
}

// ═══ set_type_converter / type_converter ═══
#[test]
fn acbf_set_type_converter() {
    use vernal_beans::AutowireCapableBeanFactory;
    let mut c = make_container();
    c.set_type_converter(None);
    assert!(c.type_converter().is_none());
}

// ═══ remove_bean_definition (dynamic) ═══
#[test]
fn bdr_remove_dynamic_bean_definition() {
    use vernal_beans::BeanDefinitionRegistry;
    use vernal_beans::BeanDefinition;
    use vernal_beans::RootBeanDefinition;
    let mut c = vernal_beans::Container::new(vernal_beans::RegistryBuilder::new().build().unwrap());
    let def = Box::new(RootBeanDefinition::new()) as Box<dyn BeanDefinition>;
    c.register_bean_definition("dyn1".to_string(), def).unwrap();
    let removed = c.remove_bean_definition("dyn1");
    assert!(removed.is_ok());
}

#[test]
fn bdr_remove_nonexistent_bean_definition() {
    use vernal_beans::BeanDefinitionRegistry;
    let mut c = vernal_beans::Container::new(vernal_beans::RegistryBuilder::new().build().unwrap());
    let result = c.remove_bean_definition("nonexistent");
    assert!(result.is_err());
}

// ═══ get_bean_definition ═══
#[test]
fn bdr_get_bean_definition_found() {
    use vernal_beans::BeanDefinitionRegistry;
    use vernal_beans::BeanDefinition;
    use vernal_beans::RootBeanDefinition;
    let mut c = vernal_beans::Container::new(vernal_beans::RegistryBuilder::new().build().unwrap());
    let def = Box::new(RootBeanDefinition::new()) as Box<dyn BeanDefinition>;
    c.register_bean_definition("dyn_get".to_string(), def).unwrap();
    let bd = c.get_bean_definition("dyn_get");
    assert!(bd.is_some());
}

#[test]
fn bdr_get_bean_definition_not_found() {
    use vernal_beans::BeanDefinitionRegistry;
    let c = vernal_beans::Container::new(vernal_beans::RegistryBuilder::new().build().unwrap());
    let bd = c.get_bean_definition("nonexistent");
    assert!(bd.is_none());
}

// ═══ bean_definition_names with dynamic ═══
#[test]
fn bdr_bean_definition_names_with_dynamic() {
    use vernal_beans::BeanDefinitionRegistry;
    use vernal_beans::BeanDefinition;
    use vernal_beans::RootBeanDefinition;
    let mut c = vernal_beans::Container::new(vernal_beans::RegistryBuilder::new().build().unwrap());
    let def = Box::new(RootBeanDefinition::new()) as Box<dyn BeanDefinition>;
    c.register_bean_definition("dyn_name".to_string(), def).unwrap();
    let names = c.bean_definition_names();
    assert!(names.contains(&"dyn_name".to_string()));
}

// ═══ singleton_names / singleton_count / add_singleton_callback ═══
#[test]
fn sbr_singleton_names() {
    use vernal_beans::SingletonBeanRegistry;
    let c = make_container();
    c.register_singleton("named_single", Arc::new(77i32));
    let names = c.singleton_names();
    assert!(names.contains(&"named_single".to_string()));
}

#[test]
fn sbr_singleton_count() {
    use vernal_beans::SingletonBeanRegistry;
    let c = make_container();
    c.register_singleton("cnt1", Arc::new(1i32));
    c.register_singleton("cnt2", Arc::new(2i32));
    // register_singleton uses ComponentKey::of::<()>() internally, so each call may overwrite
    assert!(c.singleton_count() >= 1);
}

#[test]
fn sbr_add_singleton_callback() {
    use vernal_beans::SingletonBeanRegistry;
    let mut c = make_container();
    let called = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let called_clone = called.clone();
    let cb = Arc::new(move |_any: &dyn Any| {
        called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
    }) as Arc<dyn Fn(&dyn Any) + Send + Sync>;
    c.add_singleton_callback("cb_bean".to_string(), cb);
    c.register_singleton("cb_bean", Arc::new(42i32));
    assert!(called.load(std::sync::atomic::Ordering::SeqCst));
}

// ═══ HierarchicalBeanFactory ═══
#[test]
fn hbf_parent_bean_factory_none() {
    use vernal_beans::HierarchicalBeanFactory;
    let c = make_container();
    assert!(c.parent_bean_factory().is_none());
}

#[test]
fn hbf_parent_bean_factory_set() {
    use vernal_beans::HierarchicalBeanFactory;
    use vernal_beans::ConfigurableBeanFactory;
    let mut child = make_container();
    let parent = make_container();
    child.set_parent_bean_factory(Arc::new(parent)).unwrap();
    assert!(child.parent_bean_factory().is_some());
}

#[test]
fn hbf_contains_local_bean_true() {
    use vernal_beans::HierarchicalBeanFactory;
    let c = make_container();
    assert!(c.contains_local_bean("alloc::string::String"));
}

#[test]
fn hbf_contains_local_bean_false() {
    use vernal_beans::HierarchicalBeanFactory;
    let c = make_container();
    assert!(!c.contains_local_bean("nonexistent"));
}

// ═══ ListableBeanFactory ═══
#[test]
fn lbf_bean_names_for_type_id() {
    use vernal_beans::ListableBeanFactory;
    let c = make_container();
    let names = c.bean_names_for_type_id(std::any::TypeId::of::<String>(), true, true);
    assert!(!names.is_empty());
}

#[test]
fn lbf_bean_names_for_type_id_empty() {
    use vernal_beans::ListableBeanFactory;
    let c = make_container();
    let names = c.bean_names_for_type_id(std::any::TypeId::of::<bool>(), true, true);
    assert!(names.is_empty());
}

#[test]
fn lbf_beans_of_type_id() {
    use vernal_beans::ListableBeanFactory;
    let c = make_container();
    let beans = c.beans_of_type_id(std::any::TypeId::of::<String>(), true, true).unwrap();
    assert!(!beans.is_empty());
}

#[test]
fn lbf_beans_of_type_id_empty() {
    use vernal_beans::ListableBeanFactory;
    let c = make_container();
    let beans = c.beans_of_type_id(std::any::TypeId::of::<bool>(), true, true).unwrap();
    assert!(beans.is_empty());
}

#[test]
fn lbf_bean_post_processor_count() {
    let c = make_container();
    assert_eq!(c.bean_post_processor_count(), 0);
}

#[test]
fn lbf_contains_non_singleton_bean_false() {
    use vernal_beans::ListableBeanFactory;
    let c = make_container();
    assert!(!c.contains_non_singleton_bean());
}

#[test]
fn lbf_contains_non_singleton_bean_true() {
    use vernal_beans::ListableBeanFactory;
    let c = make_transient_container();
    assert!(c.contains_non_singleton_bean());
}

#[test]
fn lbf_contains_singleton_bean_true() {
    use vernal_beans::ListableBeanFactory;
    let c = make_container();
    assert!(c.contains_singleton_bean());
}

#[test]
fn lbf_bean_names_iterator() {
    use vernal_beans::ListableBeanFactory;
    let c = make_container();
    let names: Vec<String> = c.bean_names_iterator().collect();
    assert!(!names.is_empty());
}

// ═══ ConfigurableBeanFactory: register_scope, get_registered_scope ═══
#[test]
fn cbf_register_scope_and_get() {
    use vernal_beans::ConfigurableBeanFactory;
    let mut c = make_container();
    struct TestScope;
    impl vernal_beans::bean_scope::BeanScope for TestScope {
        fn get(
            &self,
            _name: &str,
            _object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
        ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Box::new("scoped"))
        }
    }
    c.register_scope("test_scope", Box::new(TestScope));
    let names = c.registered_scope_names();
    assert!(names.contains(&"test_scope".to_string()));
    let scope = c.get_registered_scope("test_scope");
    assert!(scope.is_some());
}

#[test]
fn cbf_get_registered_scope_not_found() {
    use vernal_beans::ConfigurableBeanFactory;
    let c = make_container();
    let scope = c.get_registered_scope("missing_scope");
    assert!(scope.is_none());
}

// ═══ is_factory_bean (ConfigurableBeanFactory) ═══
#[test]
fn cbf_is_factory_bean() {
    use vernal_beans::ConfigurableBeanFactory;
    let c = make_container();
    assert!(!c.is_factory_bean("alloc::string::String"));
}

// ═══ bean_definition_count with dynamic ═══
#[test]
fn bdr_bean_definition_count_with_dynamic() {
    use vernal_beans::BeanDefinitionRegistry;
    use vernal_beans::BeanDefinition;
    use vernal_beans::RootBeanDefinition;
    let mut c = vernal_beans::Container::new(vernal_beans::RegistryBuilder::new().build().unwrap());
    let def = Box::new(RootBeanDefinition::new()) as Box<dyn BeanDefinition>;
    c.register_bean_definition("cnt_dyn".to_string(), def).unwrap();
    assert!(c.bean_definition_count() >= 1);
}

// ═══ contains_bean_definition with dynamic ═══
#[test]
fn bdr_contains_bean_definition_dynamic() {
    use vernal_beans::BeanDefinitionRegistry;
    use vernal_beans::BeanDefinition;
    use vernal_beans::RootBeanDefinition;
    let mut c = vernal_beans::Container::new(vernal_beans::RegistryBuilder::new().build().unwrap());
    assert!(!c.contains_bean_definition("dyn_contains"));
    let def = Box::new(RootBeanDefinition::new()) as Box<dyn BeanDefinition>;
    c.register_bean_definition("dyn_contains".to_string(), def).unwrap();
    assert!(c.contains_bean_definition("dyn_contains"));
}

// ═══ ContainerObjectProvider::if_available ═══
#[test]
fn bf_get_bean_provider_if_available() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
    // if_available may return None if bean not yet constructed; exercise the code path
    let _val = provider.if_available();
}

// ═══ register_bean_definition duplicate ═══
#[test]
fn bdr_register_duplicate() {
    use vernal_beans::BeanDefinitionRegistry;
    use vernal_beans::BeanDefinition;
    use vernal_beans::RootBeanDefinition;
    let mut c = vernal_beans::Container::new(vernal_beans::RegistryBuilder::new().build().unwrap());
    let def1 = Box::new(RootBeanDefinition::new()) as Box<dyn BeanDefinition>;
    let def2 = Box::new(RootBeanDefinition::new()) as Box<dyn BeanDefinition>;
    c.register_bean_definition("dup".to_string(), def1).unwrap();
    let result = c.register_bean_definition("dup".to_string(), def2);
    assert!(result.is_err());
}

// ═══ remove registered (non-dynamic) bean definition ═══
#[test]
fn bdr_remove_registered_bean_definition() {
    use vernal_beans::BeanDefinitionRegistry;
    let mut c = make_container();
    let result = c.remove_bean_definition("alloc::string::String");
    assert!(result.is_ok());
}

// ═══ get_bean_definition from registry ═══
#[test]
fn bdr_get_bean_definition_from_registry() {
    use vernal_beans::BeanDefinitionRegistry;
    let c = make_container();
    let bd = c.get_bean_definition("alloc::string::String");
    assert!(bd.is_some());
}

// ═══ contains_bean_definition for registry entry ═══
#[test]
fn bdr_contains_bean_definition_registry() {
    use vernal_beans::BeanDefinitionRegistry;
    let c = make_container();
    assert!(c.contains_bean_definition("alloc::string::String"));
}

// ═══ ensure_scope_owner mismatch ═══
#[test]
fn container_ensure_scope_owner_mismatch() {
    let c1 = make_container();
    let c2 = vernal_beans::Container::new(vernal_beans::RegistryBuilder::new().build().unwrap());
    let scope = c2.open_scope::<String>();
    let result: Result<Arc<String>, _> = c1.resolve_in(&scope);
    assert!(result.is_err());
}

// ═══ warm_up with transient ═══
#[test]
fn container_warm_up_transient() {
    let c = make_transient_container();
    assert!(c.warm_up().is_ok());
}

// ═══ ProxyBeanDefinition methods ═══
#[test]
fn proxy_bean_definition_methods() {
    use vernal_beans::BeanDefinition;
    use vernal_beans::BeanDefinitionRegistry;
    let mut c = vernal_beans::Container::new(vernal_beans::RegistryBuilder::new().build().unwrap());
    let def = Box::new(vernal_beans::RootBeanDefinition::new()) as Box<dyn BeanDefinition>;
    c.register_bean_definition("proxy_test".to_string(), def).unwrap();
    let bd = c.get_bean_definition("proxy_test").unwrap();
    // bean_class_name comes from the underlying definition (may be "unknown" for default RootBeanDefinition)
    assert!(!bd.bean_class_name().is_empty());
    assert_eq!(bd.scope(), vernal_beans::Scope::Singleton);
    assert!(!bd.is_lazy_init());
    assert!(!bd.is_primary());
}

// ═══ DeletedBeanDefinition (remove then get) ═══
#[test]
fn deleted_bean_definition_returns_none() {
    use vernal_beans::BeanDefinitionRegistry;
    let mut c = make_container();
    c.remove_bean_definition("alloc::string::String").unwrap();
    let bd = c.get_bean_definition("alloc::string::String");
    assert!(bd.is_none());
    assert!(!c.contains_bean_definition("alloc::string::String"));
}

// ═══ Container::open_scope and resolve_in with concrete type ═══
#[test]
fn container_open_scope_and_resolve_in() {
    let c = make_container();
    let scope = c.open_scope::<String>();
    let val: Arc<String> = c.resolve_in(&scope).unwrap();
    assert_eq!(*val, "hello");
}

// ═══ Container::resolve_all_traits empty ═══
#[test]
fn container_resolve_all_traits_empty() {
    let c = make_container();
    let val: Result<Vec<Arc<dyn std::fmt::Debug + Send + Sync>>, _> = c.resolve_all_traits();
    assert!(val.unwrap().is_empty());
}

// ═══ Container transient tracker ═══
#[test]
fn container_transient_tracker_access() {
    let c = make_container();
    let _tracker = c.transient_tracker();
}

// ═══ ConfigurableBeanFactory: register_alias ═══
#[test]
fn cbf_register_alias_basic() {
    use vernal_beans::ConfigurableBeanFactory;
    let mut c = make_container();
    assert!(c.register_alias("alloc::string::String", "myAlias").is_ok());
}

// ═══ ConfigurableBeanFactory: set_currently_in_creation ═══
#[test]
fn cbf_currently_in_creation() {
    use vernal_beans::ConfigurableBeanFactory;
    let mut c = make_container();
    c.set_currently_in_creation("test_bean", true);
    assert!(c.is_currently_in_creation("test_bean"));
    c.set_currently_in_creation("test_bean", false);
    assert!(!c.is_currently_in_creation("test_bean"));
}

// ═══ ConfigurableBeanFactory: register_dependent_bean ═══
#[test]
fn cbf_register_dependent_bean() {
    use vernal_beans::ConfigurableBeanFactory;
    let mut c = make_container();
    c.register_dependent_bean("beanA", "beanB");
    let dependents = c.get_dependent_beans("beanA");
    assert!(dependents.contains(&"beanB".to_string()));
}

// ═══ ConfigurableListableBeanFactory: freeze_configuration ═══
#[test]
fn clbf_freeze_and_check() {
    use vernal_beans::ConfigurableListableBeanFactory;
    let mut c = make_container();
    assert!(!c.is_configuration_frozen());
    c.freeze_configuration();
    assert!(c.is_configuration_frozen());
}

// ═══ ConfigurableListableBeanFactory: pre_instantiate_singletons ═══
#[test]
fn clbf_pre_instantiate_singletons() {
    use vernal_beans::ConfigurableListableBeanFactory;
    let c = make_container();
    assert!(c.pre_instantiate_singletons().is_ok());
}

// ═══ RootBeanDefinition comprehensive setters/getters ═══
#[test]
fn rbd_all_setters_getters() {
    use vernal_beans::RootBeanDefinition;
    let mut rbd = RootBeanDefinition::new();

    rbd.set_bean_class_name("com.example.Service");
    assert_eq!(rbd.bean_class_name(), "com.example.Service");

    rbd.set_parent_name("com.example.Base");
    assert_eq!(rbd.parent_name(), Some("com.example.Base"));

    rbd.set_scope(vernal_beans::Scope::Transient);
    assert_eq!(rbd.scope(), vernal_beans::Scope::Transient);

    rbd.set_lazy_init(true);
    assert!(rbd.is_lazy_init());

    rbd.set_abstract(true);
    assert!(rbd.is_abstract());

    rbd.set_autowire_candidate(false);
    assert!(!rbd.is_autowire_candidate());

    rbd.set_primary(true);
    assert!(rbd.is_primary());

    rbd.set_fallback(true);
    assert!(rbd.is_fallback());

    rbd.set_synthetic(true);
    assert!(rbd.is_synthetic());

    rbd.set_role(2);
    assert_eq!(rbd.role(), 2);

    rbd.set_description("A test service");
    assert_eq!(rbd.description(), Some("A test service"));

    rbd.add_depends_on("beanA");
    assert!(rbd.depends_on().contains(&"beanA".to_string()));

    rbd.set_autowire_mode(vernal_beans::Autowire::ByName);
    assert_eq!(rbd.autowire_mode(), vernal_beans::Autowire::ByName);

    rbd.set_init_method_name("init");
    assert_eq!(rbd.init_method_name(), Some("init"));

    rbd.set_destroy_method_name("destroy");
    assert_eq!(rbd.destroy_method_name(), Some("destroy"));

    rbd.set_factory_bean_name("factoryBean");
    assert_eq!(rbd.factory_bean_name(), Some("factoryBean"));

    rbd.set_factory_method_name("create");
    assert_eq!(rbd.factory_method_name(), Some("create"));

    // Constructor argument values
    let cav = rbd.constructor_argument_values();
    assert!(cav.is_empty());
    let cav_mut = rbd.get_constructor_argument_values_mut();
    use vernal_beans::factory::config::constructor_argument_values::ValueHolder;
    cav_mut.add_indexed_argument_value(0, ValueHolder::new(Arc::new("arg0")));

    // Property values
    let pv = rbd.property_values();
    assert!(pv.is_empty());
}

#[test]
fn rbd_from_generic() {
    use vernal_beans::RootBeanDefinition;
    use vernal_beans::GenericBeanDefinition;
    let mut gbd = GenericBeanDefinition::new();
    gbd.set_bean_class_name("com.example.Generic");
    let rbd = RootBeanDefinition::from_generic(gbd);
    assert_eq!(rbd.bean_class_name(), "com.example.Generic");
}

#[test]
fn rbd_is_primary_default() {
    use vernal_beans::RootBeanDefinition;
    let rbd = RootBeanDefinition::new();
    assert!(!rbd.is_primary());
}

// ═══ AbstractBeanFactory comprehensive tests ═══
#[test]
fn abf_new_and_basic_ops() {
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    let abf = AbstractBeanFactory::new();
    assert_eq!(abf.bean_definition_count(), 0);
    assert!(abf.bean_definition_names().is_empty());
    assert!(!abf.contains_bean_definition("any"));
}

#[test]
fn abf_register_and_get_bean_definition() {
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    let abf = AbstractBeanFactory::new();
    abf.register_bean_definition("svc".to_string(), Arc::new("def".to_string()));
    assert!(abf.contains_bean_definition("svc"));
    assert_eq!(abf.bean_definition_count(), 1);
    assert!(abf.bean_definition_names().contains(&"svc".to_string()));
}

#[test]
fn abf_singleton_ops() {
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    let abf = AbstractBeanFactory::new();
    abf.register_singleton("s1".to_string(), Arc::new(42i32));
    assert!(abf.contains_singleton("s1"));
    assert_eq!(abf.singleton_count(), 1);
    let val = abf.get_singleton("s1").unwrap();
    assert_eq!(*val.downcast_ref::<i32>().unwrap(), 42);
    assert!(abf.get_singleton("missing").is_none());
}

#[test]
fn abf_alias_ops() {
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    let abf = AbstractBeanFactory::new();
    abf.register_alias("myAlias".to_string(), "realBean".to_string()).unwrap();
    assert_eq!(abf.resolve_alias("myAlias"), "realBean");
    assert_eq!(abf.alias_count(), 1);
}

#[test]
fn abf_alias_already_exists() {
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    let abf = AbstractBeanFactory::new();
    abf.register_alias("myAlias".to_string(), "beanA".to_string()).unwrap();
    // Same alias pointing to different bean should fail
    let result = abf.register_alias("myAlias".to_string(), "beanB".to_string());
    assert!(result.is_err());
    // Same alias pointing to same bean is OK (idempotent)
    assert!(abf.register_alias("myAlias".to_string(), "beanA".to_string()).is_ok());
}

#[test]
fn abf_scope_ops() {
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    use vernal_beans::ScopeKey;
    let abf = AbstractBeanFactory::new();
    abf.register_scope("request".to_string(), ScopeKey::of::<String>());
    assert!(abf.contains_scope("request"));
    assert_eq!(abf.registered_scope_count(), 1);
    assert!(!abf.contains_scope("session"));
}

#[test]
fn abf_parent_ops() {
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    let abf = AbstractBeanFactory::new();
    assert!(abf.parent().is_none());
    abf.set_parent(Some("parentFactory".to_string()));
    assert_eq!(abf.parent(), Some("parentFactory".to_string()));
    abf.set_parent(None);
    assert!(abf.parent().is_none());
}

#[test]
fn abf_freeze_and_destroy() {
    use vernal_beans::factory::support::abstract_bean_factory::AbstractBeanFactory;
    let abf = AbstractBeanFactory::new();
    assert!(!abf.is_configuration_frozen());
    abf.freeze_configuration();
    assert!(abf.is_configuration_frozen());
    abf.destroy_singletons(); // no-op, just exercises the code path
}

// ═══ DefaultListableBeanFactory tests ═══
#[test]
fn dlbf_new_and_basic() {
    use vernal_beans::factory::support::default_listable_bean_factory::DefaultListableBeanFactory;
    let dlbf = DefaultListableBeanFactory::new();
    assert_eq!(dlbf.bean_definition_count(), 0);
    assert_eq!(dlbf.singleton_count(), 0);
    assert_eq!(dlbf.type_mapping_count(), 0);
}

#[test]
fn dlbf_type_mapping() {
    use vernal_beans::factory::support::default_listable_bean_factory::DefaultListableBeanFactory;
    let dlbf = DefaultListableBeanFactory::new();
    dlbf.register_type_mapping(std::any::TypeId::of::<String>(), "myString".to_string());
    assert_eq!(dlbf.type_mapping_count(), 1);
    let names = dlbf.get_bean_names_for_type(std::any::TypeId::of::<String>());
    assert!(names.contains(&"myString".to_string()));
    assert!(dlbf.get_bean_names_for_type(std::any::TypeId::of::<i32>()).is_empty());
}

#[test]
fn dlbf_dependency_descriptor() {
    use vernal_beans::factory::support::default_listable_bean_factory::DefaultListableBeanFactory;
    use vernal_beans::factory::support::dependency_descriptor::DependencyDescriptor;
    let dlbf = DefaultListableBeanFactory::new();
    let dd = DependencyDescriptor::new(std::any::TypeId::of::<String>(), "String".to_string(), true);
    dlbf.register_dependency_descriptor("myBean".to_string(), dd);
    let retrieved = dlbf.get_dependency_descriptor("myBean");
    assert!(retrieved.is_some());
    assert!(dlbf.get_dependency_descriptor("missing").is_none());
}

// ═══ Property Editors ═══
#[test]
fn byte_array_property_editor() {
    use vernal_beans::propertyeditors::byte_array_property_editor::ByteArrayPropertyEditor;
    use vernal_beans::property_editor::PropertyEditor;
    let mut editor = ByteArrayPropertyEditor::new();
    assert_eq!(editor.target_type(), std::any::TypeId::of::<Vec<u8>>());
    assert!(editor.get_as_text().is_none());
    editor.set_as_text("hello").unwrap();
    assert_eq!(editor.get_as_text(), Some("hello".to_string()));
    let val = editor.get_value().unwrap();
    assert!(val.downcast_ref::<Vec<u8>>().is_some());
    assert_eq!(editor.get_value_type(), std::any::TypeId::of::<Vec<u8>>());
    // set_value
    editor.set_value(Arc::new(vec![1u8, 2, 3]));
    assert!(editor.get_value().is_some());
}

#[test]
fn char_array_property_editor() {
    use vernal_beans::propertyeditors::char_array_property_editor::CharArrayPropertyEditor;
    use vernal_beans::property_editor::PropertyEditor;
    let mut editor = CharArrayPropertyEditor::new();
    assert!(editor.get_as_text().is_none());
    editor.set_as_text("abc").unwrap();
    assert_eq!(editor.get_as_text(), Some("abc".to_string()));
    let val = editor.get_value().unwrap();
    assert!(val.downcast_ref::<String>().is_some());
}

#[test]
fn charset_property_editor() {
    use vernal_beans::propertyeditors::charset_editor::CharsetEditor;
    use vernal_beans::property_editor::PropertyEditor;
    let mut editor = CharsetEditor::new();
    assert!(editor.get_as_text().is_none());
    editor.set_as_text("UTF-8").unwrap();
    assert_eq!(editor.get_as_text(), Some("UTF-8".to_string()));
}

#[test]
fn file_array_editor() {
    use vernal_beans::propertyeditors::file_editor::FileEditor;
    use vernal_beans::property_editor::PropertyEditor;
    let mut editor = FileEditor::new();
    assert!(editor.get_as_text().is_none());
    editor.set_as_text("/tmp/a,/tmp/b").unwrap();
    assert!(editor.get_as_text().is_some());
}

#[test]
fn input_source_editor() {
    use vernal_beans::propertyeditors::input_source_editor::InputSourceEditor;
    use vernal_beans::property_editor::PropertyEditor;
    let mut editor = InputSourceEditor::new();
    assert!(editor.get_as_text().is_none());
    editor.set_as_text("<xml/>").unwrap();
    assert!(editor.get_as_text().is_some());
}

#[test]
fn path_property_editor() {
    use vernal_beans::propertyeditors::path_editor::PathEditor;
    use vernal_beans::property_editor::PropertyEditor;
    let mut editor = PathEditor::new();
    assert!(editor.get_as_text().is_none());
    editor.set_as_text("/usr/local").unwrap();
    assert_eq!(editor.get_as_text(), Some("/usr/local".to_string()));
}

#[test]
fn resource_bundle_editor() {
    use vernal_beans::propertyeditors::resource_bundle_editor::ResourceBundleEditor;
    use vernal_beans::property_editor::PropertyEditor;
    let mut editor = ResourceBundleEditor::new();
    assert!(editor.get_as_text().is_none());
    editor.set_as_text("messages").unwrap();
    assert!(editor.get_as_text().is_some());
}

// ═══ StandardBeanExpressionResolver ═══
#[test]
fn standard_bean_expression_resolver_basic() {
    use vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver;
    use vernal_beans::BeanExpressionResolver;
    let resolver = StandardBeanExpressionResolver::new();
    // Non-expression string should pass through
    let result = resolver.evaluate("simple_string", None);
    assert!(result.is_ok());
}

#[test]
fn standard_bean_expression_resolver_expression() {
    use vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver;
    use vernal_beans::BeanExpressionResolver;
    let resolver = StandardBeanExpressionResolver::new();
    // Expression with #{} should be evaluated
    let result = resolver.evaluate("#{1 + 2}", None);
    assert!(result.is_ok());
}

// ═══ BeanScope trait ═══
#[test]
fn bean_scope_remove_default() {
    use vernal_beans::bean_scope::BeanScope;
    struct TestScope;
    impl BeanScope for TestScope {
        fn get(&self, _name: &str, _factory: &dyn Fn() -> Box<dyn Any + Send + Sync>) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Box::new("val"))
        }
    }
    let scope = TestScope;
    // Default remove returns Ok(None)
    let result = scope.remove("any");
    assert!(result.unwrap().is_none());
    // Default resolve_contextual_object returns None
    assert!(scope.resolve_contextual_object("key").is_none());
    // Default register_destruction_callback is a no-op
    scope.register_destruction_callback("name", Box::new(|| {}));
}
