/// 测试 InstantiationAwareBeanPostProcessor 和 DestructionAwareBeanPostProcessor 集成。
use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

fn make_container() -> vernal_beans::Container {
    let mut b = vernal_beans::RegistryBuilder::new();
    b.register(vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    b.register(vernal_beans::ComponentDefinition::singleton::<i32, _>(|_| 42i32));
    vernal_beans::Container::new(b.build().unwrap())
}

// ═══════════════════════════════════════════════════════════════════
// BeanPostProcessor 完整生命周期测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bean_post_processor_full_lifecycle() {
    use vernal_beans::BeanPostProcessor;

    struct LifecyclePP {
        before_init: AtomicU32,
        after_init: AtomicU32,
        before_instantiation: AtomicU32,
        after_instantiation: AtomicU32,
        before_destruction: AtomicU32,
    }

    impl LifecyclePP {
        fn new() -> Self {
            Self {
                before_init: AtomicU32::new(0),
                after_init: AtomicU32::new(0),
                before_instantiation: AtomicU32::new(0),
                after_instantiation: AtomicU32::new(0),
                before_destruction: AtomicU32::new(0),
            }
        }
    }

    impl BeanPostProcessor for LifecyclePP {
        fn post_process_before_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
            self.before_init.fetch_add(1, Ordering::SeqCst);
            Ok(Some(bean))
        }
        fn post_process_after_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
            self.after_init.fetch_add(1, Ordering::SeqCst);
            Ok(Some(bean))
        }
        fn post_process_before_instantiation(
            &self,
            _bean_class: &str,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
            self.before_instantiation.fetch_add(1, Ordering::SeqCst);
            Ok(None)
        }
        fn post_process_after_instantiation(
            &self,
            _bean: &dyn Any,
            _bean_name: &str,
        ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
            self.after_instantiation.fetch_add(1, Ordering::SeqCst);
            Ok(true)
        }
        fn post_process_before_destruction(
            &self,
            _bean: &dyn Any,
            _bean_name: &str,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            self.before_destruction.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        fn requires_destruction(&self, _bean: &dyn Any) -> bool {
            true
        }
    }

    let pp = LifecyclePP::new();

    // 测试 post_process_before_instantiation
    let result = pp.post_process_before_instantiation("com.example.Service", "service");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
    assert_eq!(pp.before_instantiation.load(Ordering::SeqCst), 1);

    // 测试 post_process_after_instantiation
    let bean = 42i32;
    let result = pp.post_process_after_instantiation(&bean, "service");
    assert!(result.is_ok());
    assert!(result.unwrap());
    assert_eq!(pp.after_instantiation.load(Ordering::SeqCst), 1);

    // 测试 post_process_before_initialization
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    let result = pp.post_process_before_initialization(bean, "service");
    assert!(result.is_ok());
    assert_eq!(pp.before_init.load(Ordering::SeqCst), 1);

    // 测试 post_process_after_initialization
    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    let result = pp.post_process_after_initialization(bean, "service");
    assert!(result.is_ok());
    assert_eq!(pp.after_init.load(Ordering::SeqCst), 1);

    // 测试 post_process_before_destruction
    let bean = 42i32;
    let result = pp.post_process_before_destruction(&bean, "service");
    assert!(result.is_ok());
    assert_eq!(pp.before_destruction.load(Ordering::SeqCst), 1);

    // 测试 requires_destruction
    assert!(pp.requires_destruction(&bean));
}

#[test]
fn bean_post_processor_proxy_before_instantiation() {
    use vernal_beans::BeanPostProcessor;

    struct ProxyPP;

    impl BeanPostProcessor for ProxyPP {
        fn post_process_before_instantiation(
            &self,
            _bean_class: &str,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Some(Arc::new("proxy_object".to_string())))
        }
        fn post_process_after_instantiation(
            &self,
            _bean: &dyn Any,
            _bean_name: &str,
        ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
            Ok(true)
        }
    }

    let pp = ProxyPP;
    let result = pp.post_process_before_instantiation("com.example.Service", "service");
    assert!(result.is_ok());
    let proxy = result.unwrap();
    assert!(proxy.is_some());
    assert_eq!(proxy.unwrap().downcast_ref::<String>().unwrap(), "proxy_object");
}

#[test]
fn bean_post_processor_skip_injection() {
    use vernal_beans::BeanPostProcessor;

    struct SkipInjectionPP;

    impl BeanPostProcessor for SkipInjectionPP {
        fn post_process_after_instantiation(
            &self,
            _bean: &dyn Any,
            _bean_name: &str,
        ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
            Ok(false) // 跳过属性注入
        }
    }

    let pp = SkipInjectionPP;
    let bean = 42i32;
    let result = pp.post_process_after_instantiation(&bean, "service");
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[test]
fn bean_post_processor_no_destruction() {
    use vernal_beans::BeanPostProcessor;

    struct NoDestructionPP;

    impl BeanPostProcessor for NoDestructionPP {
        fn requires_destruction(&self, _bean: &dyn Any) -> bool {
            false
        }
    }

    let pp = NoDestructionPP;
    let bean = 42i32;
    assert!(!pp.requires_destruction(&bean));
}

// ═══════════════════════════════════════════════════════════════════
// Container 生命周期集成测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn container_with_post_processor_chain() {
    use vernal_beans::BeanPostProcessor;

    struct CountingPP {
        id: u32,
        count: AtomicU32,
    }

    impl CountingPP {
        fn new(id: u32) -> Self {
            Self { id, count: AtomicU32::new(0) }
        }
    }

    impl BeanPostProcessor for CountingPP {
        fn post_process_before_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(Some(bean))
        }
        fn post_process_after_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(Some(bean))
        }
    }

    let pp1 = Arc::new(CountingPP::new(1));
    let pp2 = Arc::new(CountingPP::new(2));
    let pp3 = Arc::new(CountingPP::new(3));

    let mut b = vernal_beans::RegistryBuilder::new();
    b.register(vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    let mut c = vernal_beans::Container::new(b.build().unwrap());

    c.add_bean_post_processor(pp1.clone());
    c.add_bean_post_processor(pp2.clone());
    c.add_bean_post_processor(pp3.clone());
    assert_eq!(c.bean_post_processor_count(), 3);

    let val: Arc<String> = c.resolve().unwrap();
    assert_eq!(*val, "hello");

    assert!(pp1.count.load(Ordering::SeqCst) > 0);
    assert!(pp2.count.load(Ordering::SeqCst) > 0);
    assert!(pp3.count.load(Ordering::SeqCst) > 0);
}

#[test]
fn container_warm_up() {
    let mut b = vernal_beans::RegistryBuilder::new();
    b.register(vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    b.register(vernal_beans::ComponentDefinition::singleton::<i32, _>(|_| 42i32));
    let c = vernal_beans::Container::new(b.build().unwrap());

    assert!(c.warm_up().is_ok());

    let val: Arc<String> = c.resolve().unwrap();
    assert_eq!(*val, "hello");
}

#[test]
fn container_destroy_bean_instance() {
    use vernal_beans::AutowireCapableBeanFactory;
    let mut b = vernal_beans::RegistryBuilder::new();
    b.register(vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    let c = vernal_beans::Container::new(b.build().unwrap());

    let result = c.destroy_bean_instance("test_bean", &"test");
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════
// InitializingBean / DisposableBean / SmartInitializingSingleton
// ═══════════════════════════════════════════════════════════════════

#[test]
fn initializing_bean_trait() {
    use vernal_beans::InitializingBean;
    use vernal_beans::Aware;

    struct MyBean { initialized: bool }
    impl Aware for MyBean {}
    impl InitializingBean for MyBean {
        fn after_properties_set(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            self.initialized = true;
            Ok(())
        }
    }

    let mut bean = MyBean { initialized: false };
    assert!(!bean.initialized);
    bean.after_properties_set().unwrap();
    assert!(bean.initialized);
}

#[test]
fn disposable_bean_trait() {
    use vernal_beans::DisposableBean;
    use vernal_beans::Aware;

    struct MyDisposable { destroyed: std::sync::atomic::AtomicBool }
    impl Aware for MyDisposable {}
    impl DisposableBean for MyDisposable {
        fn destroy(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            self.destroyed.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    let bean = MyDisposable { destroyed: std::sync::atomic::AtomicBool::new(false) };
    assert!(!bean.destroyed.load(std::sync::atomic::Ordering::SeqCst));
    bean.destroy().unwrap();
    assert!(bean.destroyed.load(std::sync::atomic::Ordering::SeqCst));
}

#[test]
fn smart_initializing_singleton_trait() {
    use vernal_beans::SmartInitializingSingleton;

    struct MySingleton { instantiated: std::sync::atomic::AtomicBool }
    impl MySingleton {
        fn new() -> Self { Self { instantiated: std::sync::atomic::AtomicBool::new(false) } }
    }
    impl SmartInitializingSingleton for MySingleton {
        fn after_singletons_instantiated(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            self.instantiated.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    let singleton = MySingleton::new();
    assert!(!singleton.instantiated.load(std::sync::atomic::Ordering::SeqCst));
    singleton.after_singletons_instantiated().unwrap();
    assert!(singleton.instantiated.load(std::sync::atomic::Ordering::SeqCst));
}

// ═══════════════════════════════════════════════════════════════════
// Aware 回调测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn aware_trait() {
    use vernal_beans::Aware;
    use vernal_beans::BeanNameAware;

    struct MyAwareBean { bean_name: Option<String> }
    impl Aware for MyAwareBean {}
    impl BeanNameAware for MyAwareBean {
        fn set_bean_name(&mut self, name: &str) {
            self.bean_name = Some(name.to_string());
        }
    }

    let mut bean = MyAwareBean { bean_name: None };
    assert!(bean.bean_name.is_none());
    bean.set_bean_name("my_bean");
    assert_eq!(bean.bean_name.as_deref(), Some("my_bean"));
}

// ═══════════════════════════════════════════════════════════════════
// get_early_bean_reference 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn early_bean_reference_register_and_get() {
    let mut b = vernal_beans::RegistryBuilder::new();
    b.register(vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    let c = vernal_beans::Container::new(b.build().unwrap());

    let key = vernal_beans::ComponentKey::of::<String>();
    let early_ref: Arc<dyn Any + Send + Sync> = Arc::new("early_value".to_string());
    c.register_early_bean_reference(key.clone(), early_ref);

    let result = c.get_early_bean_reference(&key);
    assert!(result.is_some());
    assert_eq!(result.unwrap().downcast_ref::<String>().unwrap(), "early_value");
}

#[test]
fn early_bean_reference_not_found() {
    let mut b = vernal_beans::RegistryBuilder::new();
    b.register(vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    let c = vernal_beans::Container::new(b.build().unwrap());

    let key = vernal_beans::ComponentKey::of::<i32>();
    let result = c.get_early_bean_reference(&key);
    assert!(result.is_none());
}

#[test]
fn early_bean_reference_remove() {
    let mut b = vernal_beans::RegistryBuilder::new();
    b.register(vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    let c = vernal_beans::Container::new(b.build().unwrap());

    let key = vernal_beans::ComponentKey::of::<String>();
    c.register_early_bean_reference(key.clone(), Arc::new("early".to_string()));

    let removed = c.remove_early_bean_reference(&key);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().downcast_ref::<String>().unwrap(), "early");

    assert!(c.get_early_bean_reference(&key).is_none());
}

// ═══════════════════════════════════════════════════════════════════
// Autowire 模式测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn autowire_modes() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();

    // AUTOWIRE_NO (0)
    assert!(c.autowire("alloc::string::String", 0, false).is_ok());
    // AUTOWIRE_BY_NAME (1)
    assert!(c.autowire("alloc::string::String", 1, false).is_ok());
    // AUTOWIRE_BY_TYPE (2)
    assert!(c.autowire("alloc::string::String", 2, false).is_ok());
    // AUTOWIRE_CONSTRUCTOR (3)
    assert!(c.autowire("alloc::string::String", 3, false).is_ok());
    // Invalid mode
    assert!(c.autowire("alloc::string::String", 99, false).is_err());
}

#[test]
fn autowire_bean_properties_modes() {
    use vernal_beans::AutowireCapableBeanFactory;
    let c = make_container();

    let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
    // AUTOWIRE_NO (0)
    assert!(c.autowire_bean_properties(bean.clone(), 0, false).is_ok());
    // AUTOWIRE_BY_NAME (1)
    assert!(c.autowire_bean_properties(bean.clone(), 1, false).is_ok());
    // AUTOWIRE_BY_TYPE (2)
    assert!(c.autowire_bean_properties(bean, 2, false).is_ok());
}
