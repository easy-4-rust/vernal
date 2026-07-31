//! 深度测试 container.rs 的未覆盖方法。
use std::any::Any;
use std::sync::Arc;

fn make_container() -> vernal_beans::Container {
    let mut b = vernal_beans::RegistryBuilder::new();
    let _ = b.register(vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    let _ = b.register(vernal_beans::ComponentDefinition::singleton::<i32, _>(|_| 42i32));
    let _ = b.register(vernal_beans::ComponentDefinition::singleton::<f64, _>(|_| 3.14f64));
    vernal_beans::Container::new(b.build().unwrap())
}

// ═══════════════════════════════════════════════════════════════════
// Container resolve_definition 路径覆盖
// ═══════════════════════════════════════════════════════════════════

#[test]
fn container_resolve_transient() {
    let c = make_container();
    // Transient scope bean
    let val: Arc<String> = c.resolve().unwrap();
    assert_eq!(*val, "hello");
}

#[test]
fn container_resolve_singleton_cached() {
    let c = make_container();
    // First resolve
    let val1: Arc<String> = c.resolve().unwrap();
    // Second resolve should return cached
    let val2: Arc<String> = c.resolve().unwrap();
    assert!(Arc::ptr_eq(&val1, &val2));
}

#[test]
fn container_resolve_in_scope() {
    let c = make_container();
    let scope = c.open_scope::<String>();
    let val: Arc<String> = c.resolve_in(&scope).unwrap();
    assert_eq!(*val, "hello");
}

#[test]
fn container_resolve_qualified() {
    let mut b = vernal_beans::RegistryBuilder::new();
    let q = vernal_beans::Qualifier::new("primary").unwrap();
    let _ = b.register(vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "primary".to_string()).qualified(q.clone()));
    let c = vernal_beans::Container::new(b.build().unwrap());
    let val: Arc<String> = c.resolve_qualified(&q).unwrap();
    assert_eq!(*val, "primary");
}

#[test]
fn container_resolve_qualified_not_found() {
    let c = make_container();
    let q = vernal_beans::Qualifier::new("nonexistent").unwrap();
    let val: Result<Arc<String>, _> = c.resolve_qualified(&q);
    assert!(val.is_err());
}

#[test]
fn container_resolve_trait_no_binding() {
    let c = make_container();
    let val: Result<Arc<dyn std::fmt::Debug + Send + Sync>, _> = c.resolve_trait();
    assert!(val.is_err());
}

#[test]
fn container_resolve_all_traits_empty() {
    let c = make_container();
    let val: Result<Vec<Arc<dyn std::fmt::Debug + Send + Sync>>, _> = c.resolve_all_traits();
    assert!(val.unwrap().is_empty());
}

// ═══════════════════════════════════════════════════════════════════
// Container construct 路径覆盖
// ═══════════════════════════════════════════════════════════════════

#[test]
fn container_construct_singleton() {
    let c = make_container();
    // This triggers construct for singleton
    let val: Arc<String> = c.resolve().unwrap();
    assert_eq!(*val, "hello");
}

#[test]
fn container_construct_multiple_types() {
    let c = make_container();
    let s: Arc<String> = c.resolve().unwrap();
    let i: Arc<i32> = c.resolve().unwrap();
    let f: Arc<f64> = c.resolve().unwrap();
    assert_eq!(*s, "hello");
    assert_eq!(*i, 42);
    assert!((*f - 3.14).abs() < f64::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════
// Container BeanFactory trait coverage
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bf_get_bean_by_key() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    let bean = c.get_bean_by_key(&vernal_beans::ComponentKey::of::<String>());
    assert!(bean.is_ok());
}

#[test]
fn bf_get_bean_by_key_not_found() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    let bean = c.get_bean_by_key(&vernal_beans::ComponentKey::of::<bool>());
    assert!(bean.is_err());
}

#[test]
fn bf_get_bean_by_type_id() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    let bean = c.get_bean_by_type_id(std::any::TypeId::of::<String>());
    assert!(bean.is_ok());
}

#[test]
fn bf_get_bean_by_type_id_not_found() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    let bean = c.get_bean_by_type_id(std::any::TypeId::of::<bool>());
    assert!(bean.is_err());
}

#[test]
fn bf_contains_bean() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    assert!(c.contains_bean(&vernal_beans::ComponentKey::of::<String>()));
    assert!(!c.contains_bean(&vernal_beans::ComponentKey::of::<bool>()));
}

#[test]
fn bf_is_singleton() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    assert!(c.is_singleton(&vernal_beans::ComponentKey::of::<String>()).unwrap());
}

#[test]
fn bf_is_prototype() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    assert!(!c.is_prototype(&vernal_beans::ComponentKey::of::<String>()).unwrap());
}

#[test]
fn bf_get_type() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    let t = c.get_type(&vernal_beans::ComponentKey::of::<String>());
    assert!(t.is_ok());
}

#[test]
fn bf_get_aliases() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    let aliases = c.get_aliases(&vernal_beans::ComponentKey::of::<String>());
    assert!(aliases.is_empty());
}

#[test]
fn bf_is_type_match() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    assert!(c.is_type_match(&vernal_beans::ComponentKey::of::<String>(), std::any::TypeId::of::<String>()));
}

#[test]
fn bf_get_bean_provider() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>());
    assert!(provider.is_ok());
}

// ═══════════════════════════════════════════════════════════════════
// Container AutowireCapableBeanFactory coverage
// ═══════════════════════════════════════════════════════════════════

#[test]
fn acbf_create_bean() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let bean = c.create_bean("alloc::string::String");
    assert!(bean.is_ok());
}

#[test]
fn acbf_create_bean_not_found() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let bean = c.create_bean("nonexistent");
    assert!(bean.is_err());
}

#[test]
fn acbf_autowire_bean() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    assert!(c.autowire_bean(bean).is_ok());
}

#[test]
fn acbf_initialize_bean() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    assert!(c.initialize_bean(bean, "test_bean").is_ok());
}

#[test]
fn acbf_configure_bean() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    assert!(c.configure_bean(bean, "test_bean").is_ok());
}

#[test]
fn acbf_destroy_bean_instance() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    assert!(c.destroy_bean_instance("test_bean", &"test").is_ok());
}

#[test]
fn acbf_autowire_modes() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    assert!(c.autowire("alloc::string::String", 0, false).is_ok());
    assert!(c.autowire("alloc::string::String", 1, false).is_ok());
    assert!(c.autowire("alloc::string::String", 2, false).is_ok());
    assert!(c.autowire("alloc::string::String", 3, false).is_ok());
    assert!(c.autowire("alloc::string::String", 99, false).is_err());
}

#[test]
fn acbf_resolve_named_bean() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    assert!(c.resolve_named_bean(std::any::TypeId::of::<String>()).is_ok());
}

#[test]
fn acbf_resolve_named_bean_not_found() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = vernal_beans::Container::new(vernal_beans::RegistryBuilder::new().build().unwrap());
    assert!(c.resolve_named_bean(std::any::TypeId::of::<String>()).is_err());
}

#[test]
fn acbf_type_converter() {
    use vernal_beans::AutowireCapableBeanFactory;
    let mut c = make_container();
    assert!(c.type_converter().is_none());
    c.set_type_converter(None);
}

// ═══════════════════════════════════════════════════════════════════
// Container BeanDefinitionRegistry coverage
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bdr_register_and_remove() {
    use vernal_beans::BeanDefinitionRegistry;
    use vernal_beans::BeanDefinition;
    use vernal_beans::RootBeanDefinition;
    let mut c = vernal_beans::Container::new(vernal_beans::RegistryBuilder::new().build().unwrap());
    let def = Box::new(RootBeanDefinition::new()) as Box<dyn BeanDefinition>;
    c.register_bean_definition("dyn".to_string(), def).unwrap();
    assert!(c.contains_bean_definition("dyn"));
    assert!(c.remove_bean_definition("dyn").is_ok());
    assert!(!c.contains_bean_definition("dyn"));
}

#[test]
fn bdr_count_and_names() {
    use vernal_beans::BeanDefinitionRegistry;
    let c = make_container();
    assert!(c.bean_definition_count() >= 2);
    let names = c.bean_definition_names();
    assert!(names.len() >= 2);
}

#[test]
fn bdr_get_bean_definition() {
    use vernal_beans::BeanDefinitionRegistry;
    let c = make_container();
    let def = c.get_bean_definition("alloc::string::String");
    assert!(def.is_some());
}

// ═══════════════════════════════════════════════════════════════════
// Container HierarchicalBeanFactory coverage
// ═══════════════════════════════════════════════════════════════════

#[test]
fn hbf_parent_none() {
    use vernal_beans::HierarchicalBeanFactory;
    let c = make_container();
    assert!(c.parent_bean_factory().is_none());
}

#[test]
fn hbf_contains_local_bean() {
    use vernal_beans::HierarchicalBeanFactory;
    let c = make_container();
    assert!(!c.contains_local_bean("nonexistent"));
}

// ═══════════════════════════════════════════════════════════════════
// Container ListableBeanFactory coverage
// ═══════════════════════════════════════════════════════════════════

#[test]
fn lbf_bean_definition_count() {
    use vernal_beans::ListableBeanFactory;
    let c = make_container();
    assert!(c.bean_definition_count() >= 2);
}

#[test]
fn lbf_bean_definition_names() {
    use vernal_beans::ListableBeanFactory;
    let c = make_container();
    let names = c.bean_definition_names();
    assert!(names.len() >= 2);
}

#[test]
fn lbf_contains_bean_definition() {
    use vernal_beans::ListableBeanFactory;
    let c = make_container();
    assert!(c.contains_bean_definition("alloc::string::String"));
    assert!(!c.contains_bean_definition("nonexistent"));
}

#[test]
fn lbf_bean_names_for_type() {
    use vernal_beans::ListableBeanFactory;
    let c = make_container();
    let names = c.bean_names_for_type_id(std::any::TypeId::of::<String>(), true, true);
    assert!(!names.is_empty());
}

#[test]
fn lbf_bean_post_processor_count() {
    let c = make_container();
    assert_eq!(c.bean_post_processor_count(), 0);
}

#[test]
fn lbf_contains_singleton_bean() {
    use vernal_beans::ListableBeanFactory;
    let c = make_container();
    assert!(c.contains_singleton_bean());
}

#[test]
fn lbf_contains_non_singleton_bean() {
    use vernal_beans::ListableBeanFactory;
    let c = make_container();
    assert!(!c.contains_non_singleton_bean());
}

#[test]
fn lbf_bean_names_iterator() {
    use vernal_beans::ListableBeanFactory;
    let c = make_container();
    let names: Vec<String> = c.bean_names_iterator().collect();
    assert!(names.len() >= 2);
}

// ═══════════════════════════════════════════════════════════════════
// Container ConfigurableBeanFactory coverage
// ═══════════════════════════════════════════════════════════════════

#[test]
fn cbf_register_scope() {
    use vernal_beans::ConfigurableBeanFactory;
    use vernal_beans::bean_scope::BeanScope;
    let mut c = make_container();
    struct TestScope;
    impl BeanScope for TestScope {
        fn get(&self, _n: &str, _f: &dyn Fn() -> Box<dyn Any + Send + Sync>) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Box::new("scope_val"))
        }
    }
    c.register_scope("test_scope", Box::new(TestScope));
    let names = c.registered_scope_names();
    assert!(names.contains(&"test_scope".to_string()));
}

#[test]
fn cbf_register_alias() {
    use vernal_beans::ConfigurableBeanFactory;
    let mut c = make_container();
    assert!(c.register_alias("alloc::string::String", "myAlias").is_ok());
}

#[test]
fn cbf_set_currently_in_creation() {
    use vernal_beans::ConfigurableBeanFactory;
    let mut c = make_container();
    c.set_currently_in_creation("test_bean", true);
    assert!(c.is_currently_in_creation("test_bean"));
    c.set_currently_in_creation("test_bean", false);
    assert!(!c.is_currently_in_creation("test_bean"));
}

#[test]
fn cbf_register_dependent_bean() {
    use vernal_beans::ConfigurableBeanFactory;
    let mut c = make_container();
    c.register_dependent_bean("beanA", "beanB");
    let dependents = c.get_dependent_beans("beanA");
    assert!(dependents.contains(&"beanB".to_string()));
    let deps = c.get_dependencies_for_bean("beanB");
    assert!(deps.contains(&"beanA".to_string()));
}

// ═══════════════════════════════════════════════════════════════════
// Container ConfigurableListableBeanFactory coverage
// ═══════════════════════════════════════════════════════════════════

#[test]
fn clbf_freeze_and_check() {
    use vernal_beans::ConfigurableListableBeanFactory;
    let mut c = make_container();
    assert!(!c.is_configuration_frozen());
    c.freeze_configuration();
    assert!(c.is_configuration_frozen());
}

#[test]
fn clbf_pre_instantiate_singletons() {
    use vernal_beans::ConfigurableListableBeanFactory;
    let c = make_container();
    assert!(c.pre_instantiate_singletons().is_ok());
}

#[test]
fn clbf_ignore_dependency_type() {
    use vernal_beans::ConfigurableListableBeanFactory;
    let mut c = make_container();
    c.ignore_dependency_type(std::any::TypeId::of::<String>());
}

// ═══════════════════════════════════════════════════════════════════
// Container SingletonBeanRegistry coverage
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sbr_register_get_contains() {
    use vernal_beans::SingletonBeanRegistry;
    let c = make_container();
    c.register_singleton("my_singleton", Arc::new(99i32));
    assert!(c.contains_singleton("my_singleton"));
    let v = c.get_singleton("my_singleton");
    assert!(v.is_some());
    assert_eq!((*v.unwrap()).downcast_ref::<i32>().copied(), Some(99));
}

#[test]
fn sbr_names_and_count() {
    use vernal_beans::SingletonBeanRegistry;
    let c = make_container();
    let _ = c.resolve::<Arc<String>>();
    let _ = c.singleton_names();
    let _ = c.singleton_count();
}

#[test]
fn sbr_mutex() {
    use vernal_beans::SingletonBeanRegistry;
    let c = make_container();
    let m1 = c.singleton_mutex();
    let m2 = c.singleton_mutex();
    assert!(Arc::ptr_eq(&m1, &m2));
}

// ═══════════════════════════════════════════════════════════════════
// Container warm_up coverage
// ═══════════════════════════════════════════════════════════════════

#[test]
fn container_warm_up() {
    let c = make_container();
    assert!(c.warm_up().is_ok());
    assert!(c.unused_definitions().is_empty());
}

// ═══════════════════════════════════════════════════════════════════
// Container PostProcessor coverage
// ═══════════════════════════════════════════════════════════════════

#[test]
fn container_post_processor() {
    use vernal_beans::BeanPostProcessor;
    let mut c = make_container();
    struct PP; impl BeanPostProcessor for PP {}
    c.add_bean_post_processor(Arc::new(PP));
    c.add_bean_post_processor(Arc::new(PP));
    assert_eq!(c.bean_post_processor_count(), 2);
}

// ═══════════════════════════════════════════════════════════════════
// Container scope coverage
// ═══════════════════════════════════════════════════════════════════

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
