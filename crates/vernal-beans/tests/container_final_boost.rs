//! Container deep coverage tests — no test hooks, pure API.
use std::any::Any;
use std::sync::Arc;
use vernal_beans::{
    bean_definition::BeanDefinition, bean_factory::BeanFactory,
    autowire_capable_bean_factory::AutowireCapableBeanFactory,
    bean_definition_registry::BeanDefinitionRegistry,
    ComponentDefinition, ComponentKey, Container, RegistryBuilder,
};

fn make_container() -> Container {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32));
    Container::new(b.build().unwrap())
}

#[test]
fn get_type_ok() {
    let c = make_container();
    assert_eq!(c.get_type(&ComponentKey::of::<String>()).unwrap(), Some("alloc::string::String"));
    assert!(c.get_type(&ComponentKey::of::<f64>()).is_err());
}

#[test]
fn get_aliases_empty() {
    let c = make_container();
    assert!(c.get_aliases(&ComponentKey::of::<String>()).is_empty());
}

#[test]
fn is_type_match() {
    let c = make_container();
    assert!(c.is_type_match(&ComponentKey::of::<String>(), std::any::TypeId::of::<String>()));
    assert!(!c.is_type_match(&ComponentKey::of::<String>(), std::any::TypeId::of::<i32>()));
}

#[test]
fn get_bean_provider() {
    let c = make_container();
    let p = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
    let _ = p.get();
}

#[test]
fn create_bean_err() {
    let c = make_container();
    assert!(AutowireCapableBeanFactory::create_bean(&c, "unknown").is_err());
}

#[test]
fn autowire_bean_passthrough() {
    let c = make_container();
    let bean = Arc::new("x".to_string()) as Arc<dyn Any + Send + Sync>;
    assert!(AutowireCapableBeanFactory::autowire_bean(&c, bean).is_ok());
}

#[test]
fn initialize_bean_passthrough() {
    let c = make_container();
    let bean = Arc::new("x".to_string()) as Arc<dyn Any + Send + Sync>;
    assert!(AutowireCapableBeanFactory::initialize_bean(&c, bean, "test").is_ok());
}

#[test]
fn configure_bean_passthrough() {
    let c = make_container();
    let bean = Arc::new("x".to_string()) as Arc<dyn Any + Send + Sync>;
    assert!(AutowireCapableBeanFactory::configure_bean(&c, bean, "test").is_ok());
}

#[test]
fn destroy_instance() {
    let c = make_container();
    assert!(AutowireCapableBeanFactory::destroy_bean_instance(&c, "test", &"x").is_ok());
}

#[test]
fn autowire_modes() {
    let c = make_container();
    assert!(AutowireCapableBeanFactory::autowire(&c, "alloc::string::String", 0, false).is_ok());
    assert!(AutowireCapableBeanFactory::autowire(&c, "alloc::string::String", 1, false).is_ok());
    assert!(AutowireCapableBeanFactory::autowire(&c, "alloc::string::String", 2, false).is_ok());
    assert!(AutowireCapableBeanFactory::autowire(&c, "alloc::string::String", 3, false).is_ok());
    assert!(AutowireCapableBeanFactory::autowire(&c, "alloc::string::String", 99, false).is_err());
}

#[test]
fn autowire_bean_properties_modes() {
    let c = make_container();
    let bean = Arc::new(42i32) as Arc<dyn Any + Send + Sync>;
    assert!(AutowireCapableBeanFactory::autowire_bean_properties(&c, bean.clone(), 0, false).is_ok());
    assert!(AutowireCapableBeanFactory::autowire_bean_properties(&c, bean.clone(), 1, false).is_ok());
    assert!(AutowireCapableBeanFactory::autowire_bean_properties(&c, bean, 2, false).is_ok());
}

#[test]
fn resolve_dependency_found() {
    let c = make_container();
    let desc = vernal_beans::DependencyDescriptor::for_field(
        std::any::TypeId::of::<String>(),
        "String",
    );
    assert!(AutowireCapableBeanFactory::resolve_dependency(&c, &desc, None).is_ok());
}

#[test]
fn resolve_dependency_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let desc = vernal_beans::DependencyDescriptor::for_field(
        std::any::TypeId::of::<String>(),
        "String",
    );
    assert!(AutowireCapableBeanFactory::resolve_dependency(&c, &desc, None).is_err());
}

#[test]
fn resolve_dependency_multiple() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
    b.register(ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
        .qualified(vernal_beans::Qualifier::new("q2").unwrap()));
    let c = Container::new(b.build().unwrap());
    let desc = vernal_beans::DependencyDescriptor::for_field(
        std::any::TypeId::of::<String>(),
        "String",
    );
    assert!(AutowireCapableBeanFactory::resolve_dependency(&c, &desc, None).is_err());
}

#[test]
fn resolve_named_bean_found() {
    let c = make_container();
    assert!(AutowireCapableBeanFactory::resolve_named_bean(&c, std::any::TypeId::of::<String>()).is_ok());
}

#[test]
fn resolve_named_bean_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(AutowireCapableBeanFactory::resolve_named_bean(&c, std::any::TypeId::of::<String>()).is_err());
}

#[test]
fn registry_register_and_remove() {
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    let def = Box::new(RootBeanDefinition::new()) as Box<dyn BeanDefinition>;
    BeanDefinitionRegistry::register_bean_definition(&mut c, "dyn".to_string(), def).unwrap();
    assert!(BeanDefinitionRegistry::contains_bean_definition(&c, "dyn"));
    assert!(BeanDefinitionRegistry::remove_bean_definition(&mut c, "dyn").is_ok());
    assert!(!BeanDefinitionRegistry::contains_bean_definition(&c, "dyn"));
}

#[test]
fn registry_count_and_names() {
    let c = make_container();
    assert_eq!(BeanDefinitionRegistry::bean_definition_count(&c), 2);
    assert!(BeanDefinitionRegistry::bean_definition_names(&c).len() >= 2);
}
