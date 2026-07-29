use std::sync::Arc;
use vernal_beans::{
    AbstractBeanDefinition, AbstractBeanFactory, AutowireCandidateResolver, BeanDefinition,
    BeanReferenceResolver, ComponentDefinition, ConstructorResolver, Container,
    DefaultListableBeanFactory, DefaultParameterNameDiscoverer, DisposableBean,
    DisposableBeanAdapter, ParameterNameDiscoverer, RegistryBuilder, SimpleInstantiationStrategy,
};

#[test]
fn abf_new() {
    let f = AbstractBeanFactory::new();
    assert!(f.get_bean_post_processors().is_empty());
}

#[test]
fn abf_add_processor() {
    use vernal_beans::bean_post_processor::BeanPostProcessor;
    struct P;
    impl BeanPostProcessor for P {
        fn post_process_before_initialization(
            &self,
            _b: Arc<dyn std::any::Any + Send + Sync>,
            _n: &str,
        ) -> Result<
            Option<Arc<dyn std::any::Any + Send + Sync>>,
            Box<dyn std::error::Error + Send + Sync>,
        > {
            Ok(None)
        }
        fn post_process_after_initialization(
            &self,
            _b: Arc<dyn std::any::Any + Send + Sync>,
            _n: &str,
        ) -> Result<
            Option<Arc<dyn std::any::Any + Send + Sync>>,
            Box<dyn std::error::Error + Send + Sync>,
        > {
            Ok(None)
        }
    }
    let mut f = AbstractBeanFactory::new();
    f.add_bean_post_processor(Arc::new(P));
    assert!(f.get_bean_post_processors().len() == 1);
}

#[test]
fn abf_ignore_dep() {
    let mut f = AbstractBeanFactory::new();
    f.ignore_dependency_type(std::any::TypeId::of::<String>());
    let s_tid = std::any::TypeId::of::<String>();
    let i_tid = std::any::TypeId::of::<i32>();
    assert!(f.is_ignored_dependency_type(&s_tid));
    assert!(!f.is_ignored_dependency_type(&i_tid));
}

#[test]
fn dlbf_new() {
    let _f = DefaultListableBeanFactory::new(RegistryBuilder::new().build().unwrap());
}

#[test]
fn dlbf_pre_instantiate() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }));
    let f = DefaultListableBeanFactory::new(b.build().unwrap());
    assert!(f.pre_instantiate_singletons().is_ok());
}

#[test]
fn cr_new() {
    let _r = ConstructorResolver::new(SimpleInstantiationStrategy::new());
}

#[test]
fn acr_default() {
    #[derive(Debug)]
    struct R;
    impl AutowireCandidateResolver for R {}
    let d = AbstractBeanDefinition::new();
    let bd: &dyn BeanDefinition = &d;
    assert!(R.is_autowire_candidate(bd, None));
}

#[test]
fn dba_new_destroy() {
    struct T;
    impl vernal_beans::aware::Aware for T {}
    impl DisposableBean for T {
        fn destroy(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
    }
    let a = DisposableBeanAdapter::new(Arc::new(T) as Arc<dyn DisposableBean>, "b");
    let _ = a.destroy();
}

#[test]
fn brr_new_and_resolve() {
    use vernal_beans::BeanFactory;
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let r = BeanReferenceResolver::new(Arc::new(c));
    assert!(r.resolve_by_name(std::any::type_name::<String>()).is_ok());
    let _ = r.resolve_by_name("nope");
}

#[test]
fn brr_contains() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    let r = BeanReferenceResolver::new(Arc::new(c));
    // Just call the method
    let _ = r.contains_bean(std::any::type_name::<String>());
    let _ = r.contains_bean("nope");
}

#[test]
fn pnd_default() {
    let d = DefaultParameterNameDiscoverer::new();
    let m = Box::new(String::from("m")) as Box<dyn std::any::Any>;
    assert!(d.get_parameter_names(m.as_ref()).is_empty());
}

#[test]
fn pnd_trait() {
    let d: Arc<dyn ParameterNameDiscoverer> = Arc::new(DefaultParameterNameDiscoverer::new());
    let m = Box::new(String::from("t")) as Box<dyn std::any::Any>;
    assert!(d.get_parameter_names(m.as_ref()).is_empty());
}

#[test]
fn fbd_ops() {
    use vernal_beans::factory_bean::FactoryBean;
    use vernal_beans::factory_bean_delegate::FactoryBeanDelegate;
    struct F;
    impl FactoryBean for F {
        fn get_object(
            &self,
        ) -> Result<Arc<dyn std::any::Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(Arc::new("v".to_string()))
        }
        fn get_object_type(&self) -> Option<std::any::TypeId> {
            Some(std::any::TypeId::of::<String>())
        }
        fn is_singleton(&self) -> bool {
            true
        }
    }
    assert!(FactoryBeanDelegate::is_singleton(&F));
    assert!(FactoryBeanDelegate::get_object_type(&F).is_some());
    assert!(FactoryBeanDelegate::get_object_from_factory_bean(&F).is_ok());
    assert!(FactoryBeanDelegate::should_extract_object("name"));
    assert!(!FactoryBeanDelegate::should_extract_object("&name"));
}
