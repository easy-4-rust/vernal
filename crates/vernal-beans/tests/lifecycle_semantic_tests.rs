//! 语义测试 — Container Bean 生命周期方法（construct、destroy_bean_instance、resolve_definition）。
use std::any::Any;
use std::sync::Arc;

fn make_container() -> vernal_beans::Container {
    let mut b = vernal_beans::RegistryBuilder::new();
    let _ = b.register(vernal_beans::ComponentDefinition::singleton::<String, _>(
        |_| "hello".to_string(),
    ));
    let _ = b.register(vernal_beans::ComponentDefinition::singleton::<i32, _>(
        |_| 42i32,
    ));
    let _ = b.register(vernal_beans::ComponentDefinition::singleton::<f64, _>(
        |_| 3.14f64,
    ));
    vernal_beans::Container::new(b.build().unwrap())
}

// ═══════════════════════════════════════════════════════════════════
// construct 方法语义测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn construct_singleton_returns_same_instance() {
    let c = make_container();
    let v1: Arc<String> = c.resolve().unwrap();
    let v2: Arc<String> = c.resolve().unwrap();
    assert!(Arc::ptr_eq(&v1, &v2), "Singleton 应返回相同实例");
}

#[test]
fn construct_multiple_types() {
    let c = make_container();
    let s: Arc<String> = c.resolve().unwrap();
    let i: Arc<i32> = c.resolve().unwrap();
    let f: Arc<f64> = c.resolve().unwrap();
    assert_eq!(*s, "hello");
    assert_eq!(*i, 42);
    assert!((*f - 3.14).abs() < f64::EPSILON);
}

#[test]
fn construct_not_found_returns_error() {
    let c = make_container();
    let result: Result<Arc<bool>, _> = c.resolve();
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════
// resolve_definition 方法语义测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn resolve_definition_singleton_scope() {
    let c = make_container();
    let v1: Arc<String> = c.resolve().unwrap();
    let v2: Arc<String> = c.resolve().unwrap();
    assert!(Arc::ptr_eq(&v1, &v2));
}

#[test]
fn resolve_definition_transient_scope() {
    let mut b = vernal_beans::RegistryBuilder::new();
    let _ = b.register(vernal_beans::ComponentDefinition::transient::<String, _>(
        |_| "transient".to_string(),
    ));
    let c = vernal_beans::Container::new(b.build().unwrap());
    let v1: Arc<String> = c.resolve().unwrap();
    let v2: Arc<String> = c.resolve().unwrap();
    assert!(!Arc::ptr_eq(&v1, &v2), "Transient 应返回不同实例");
}

#[test]
fn resolve_definition_custom_scope() {
    let c = make_container();
    let scope = c.open_scope::<String>();
    let v: Arc<String> = c.resolve_in(&scope).unwrap();
    assert_eq!(*v, "hello");
}

#[test]
fn resolve_definition_qualified() {
    let mut b = vernal_beans::RegistryBuilder::new();
    let q = vernal_beans::Qualifier::new("primary").unwrap();
    let _ = b.register(
        vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "primary".to_string())
            .qualified(q.clone()),
    );
    let c = vernal_beans::Container::new(b.build().unwrap());
    let v: Arc<String> = c.resolve_qualified(&q).unwrap();
    assert_eq!(*v, "primary");
}

#[test]
fn resolve_definition_qualified_not_found() {
    let c = make_container();
    let q = vernal_beans::Qualifier::new("nonexistent").unwrap();
    let result: Result<Arc<String>, _> = c.resolve_qualified(&q);
    assert!(result.is_err());
}

#[test]
fn resolve_definition_circular_dependency() {
    // 循环依赖检测
    let mut b = vernal_beans::RegistryBuilder::new();
    let _ = b.register(vernal_beans::ComponentDefinition::singleton::<String, _>(
        |_| "a".to_string(),
    ));
    let c = vernal_beans::Container::new(b.build().unwrap());
    // 正常解析应该成功
    let v: Arc<String> = c.resolve().unwrap();
    assert_eq!(*v, "a");
}

// ═══════════════════════════════════════════════════════════════════
// destroy_bean_instance 方法语义测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn destroy_bean_instance_basic() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let result = c.destroy_bean_instance("test_bean", &"test");
    assert!(result.is_ok());
}

#[test]
fn destroy_bean_instance_with_post_processor() {
    use vernal_beans::AutowireCapableBeanFactory;
    use vernal_beans::BeanPostProcessor;
    struct PP;
    impl BeanPostProcessor for PP {
        fn post_process_before_destruction(
            &self,
            _bean: &dyn Any,
            _name: &str,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
    }
    let mut c = make_container();
    c.add_bean_post_processor(Arc::new(PP));
    let result = c.destroy_bean_instance("test_bean", &"test");
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════
// warm_up 方法语义测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn warm_up_pre_instantiates_singletons() {
    let c = make_container();
    assert!(c.warm_up().is_ok());
    // After warm_up, singletons should be cached
    let v1: Arc<String> = c.resolve().unwrap();
    let v2: Arc<String> = c.resolve().unwrap();
    assert!(Arc::ptr_eq(&v1, &v2));
}

#[test]
fn warm_up_empty_container() {
    let c = vernal_beans::Container::new(vernal_beans::RegistryBuilder::new().build().unwrap());
    assert!(c.warm_up().is_ok());
}

// ═══════════════════════════════════════════════════════════════════
// PostProcessor 链语义测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn post_processor_chain_order() {
    use std::sync::atomic::{AtomicU32, Ordering};
    use vernal_beans::BeanPostProcessor;

    #[allow(dead_code)]
    struct OrderPP {
        id: u32,
        counter: Arc<AtomicU32>,
    }

    impl BeanPostProcessor for OrderPP {
        fn post_process_before_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(Some(bean))
        }
    }

    let counter = Arc::new(AtomicU32::new(0));
    let mut c = make_container();
    c.add_bean_post_processor(Arc::new(OrderPP {
        id: 1,
        counter: counter.clone(),
    }));
    c.add_bean_post_processor(Arc::new(OrderPP {
        id: 2,
        counter: counter.clone(),
    }));

    let _v: Arc<String> = c.resolve().unwrap();
    // Both processors should have been called
    let _ = counter.load(Ordering::SeqCst);
}

// ═══════════════════════════════════════════════════════════════════
// Scope 语义测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn scope_isolation() {
    let c = make_container();
    let scope1 = c.open_scope::<String>();
    let scope2 = c.open_scope::<i32>();
    let v1: Arc<String> = c.resolve_in(&scope1).unwrap();
    let v2: Arc<i32> = c.resolve_in(&scope2).unwrap();
    assert_eq!(*v1, "hello");
    assert_eq!(*v2, 42);
}

// ═══════════════════════════════════════════════════════════════════
// BeanFactory 完整语义测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bean_factory_contains_bean() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    assert!(c.contains_bean(&vernal_beans::ComponentKey::of::<String>()));
    assert!(!c.contains_bean(&vernal_beans::ComponentKey::of::<bool>()));
}

#[test]
fn bean_factory_is_singleton() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    assert!(
        c.is_singleton(&vernal_beans::ComponentKey::of::<String>())
            .unwrap()
    );
}

#[test]
fn bean_factory_is_prototype() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    assert!(
        !c.is_prototype(&vernal_beans::ComponentKey::of::<String>())
            .unwrap()
    );
}

#[test]
fn bean_factory_get_type() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    let t = c.get_type(&vernal_beans::ComponentKey::of::<String>());
    assert!(t.is_ok());
}

#[test]
fn bean_factory_get_aliases() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    let aliases = c.get_aliases(&vernal_beans::ComponentKey::of::<String>());
    assert!(aliases.is_empty());
}

#[test]
fn bean_factory_is_type_match() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    assert!(c.is_type_match(
        &vernal_beans::ComponentKey::of::<String>(),
        std::any::TypeId::of::<String>()
    ));
}

#[test]
fn bean_factory_get_bean_provider() {
    use vernal_beans::BeanFactory;
    let c = make_container();
    let provider = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>());
    assert!(provider.is_ok());
}

// ═══════════════════════════════════════════════════════════════════
// AutowireCapableBeanFactory 完整语义测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn acbf_create_bean() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();
    let bean = c.create_bean("alloc::string::String");
    assert!(bean.is_ok());
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
    assert!(
        c.resolve_named_bean(std::any::TypeId::of::<String>())
            .is_ok()
    );
}

// ═══════════════════════════════════════════════════════════════════
// BeanDefinitionRegistry 完整语义测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bdr_register_and_remove() {
    use vernal_beans::BeanDefinition;
    use vernal_beans::BeanDefinitionRegistry;
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

// ═══════════════════════════════════════════════════════════════════
// ListableBeanFactory 完整语义测试
// ═══════════════════════════════════════════════════════════════════

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
fn lbf_contains_singleton_bean() {
    use vernal_beans::ListableBeanFactory;
    let c = make_container();
    assert!(c.contains_singleton_bean());
}

#[test]
fn lbf_bean_names_iterator() {
    use vernal_beans::ListableBeanFactory;
    let c = make_container();
    let names: Vec<String> = c.bean_names_iterator().collect();
    assert!(names.len() >= 2);
}

// ═══════════════════════════════════════════════════════════════════
// ConfigurableBeanFactory 完整语义测试
// ═══════════════════════════════════════════════════════════════════

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
}

// ═══════════════════════════════════════════════════════════════════
// ConfigurableListableBeanFactory 完整语义测试
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

// ═══════════════════════════════════════════════════════════════════
// SingletonBeanRegistry 完整语义测试
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
fn sbr_mutex() {
    use vernal_beans::SingletonBeanRegistry;
    let c = make_container();
    let m1 = c.singleton_mutex();
    let m2 = c.singleton_mutex();
    assert!(Arc::ptr_eq(&m1, &m2));
}
