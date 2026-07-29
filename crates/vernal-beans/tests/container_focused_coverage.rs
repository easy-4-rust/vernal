use std::sync::Arc;
use vernal_beans::{
    BeanDefinitionRegistry, BeanFactory, ComponentDefinition, ComponentKey,
    ConfigurableBeanFactory, Container, RegistryBuilder,
};

fn container() -> Container {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }));
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32));
    Container::new(b.build().unwrap())
}

fn cbf(c: &mut Container) -> &mut dyn ConfigurableBeanFactory {
    c
}

// ── get_bean_definition paths ──────────────────────────────────

#[test]
fn gbd_dynamic_non_deleted() {
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    let def = Box::new(RootBeanDefinition::new()) as Box<dyn vernal_beans::BeanDefinition>;
    BeanDefinitionRegistry::register_bean_definition(&mut c, "dyn".to_string(), def).unwrap();
    assert!(BeanDefinitionRegistry::get_bean_definition(&c, "dyn").is_some());
}

#[test]
fn gbd_deleted_returns_none() {
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    let def = Box::new(RootBeanDefinition::new()) as Box<dyn vernal_beans::BeanDefinition>;
    BeanDefinitionRegistry::register_bean_definition(&mut c, "x".to_string(), def).unwrap();
    BeanDefinitionRegistry::remove_bean_definition(&mut c, "x").unwrap();
    assert!(BeanDefinitionRegistry::get_bean_definition(&c, "x").is_none());
}

#[test]
fn gbd_registry_path() {
    let c = container();
    assert!(
        BeanDefinitionRegistry::get_bean_definition(&c, std::any::type_name::<String>()).is_some()
    );
}

#[test]
fn gbd_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(BeanDefinitionRegistry::get_bean_definition(&c, "nope").is_none());
}

#[test]
fn contains_def_deleted_false() {
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    let def = Box::new(RootBeanDefinition::new()) as Box<dyn vernal_beans::BeanDefinition>;
    BeanDefinitionRegistry::register_bean_definition(&mut c, "t".to_string(), def).unwrap();
    BeanDefinitionRegistry::remove_bean_definition(&mut c, "t").unwrap();
    assert!(!BeanDefinitionRegistry::contains_bean_definition(&c, "t"));
}

// ── ConfigurableBeanFactory scopes ─────────────────────────────

struct SimpleScope;
impl vernal_beans::bean_scope::BeanScope for SimpleScope {
    fn get(
        &self,
        _name: &str,
        _factory: &dyn Fn() -> Box<dyn std::any::Any + Send + Sync>,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>
    {
        Ok(Box::new("v".to_string()))
    }
}

#[test]
fn cbf_scope_reg_list() {
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    let f = cbf(&mut c);
    f.register_scope("s1", Box::new(SimpleScope));
    assert!(f.registered_scope_names().contains(&"s1".to_string()));
}

#[test]
fn cbf_is_factory_bean() {
    let c = container();
    assert!(ConfigurableBeanFactory::is_factory_bean(&c, "&fb"));
    assert!(!ConfigurableBeanFactory::is_factory_bean(&c, "normal"));
}

#[test]
fn cbf_creation_lifecycle() {
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    let f = cbf(&mut c);
    assert!(!ConfigurableBeanFactory::is_currently_in_creation(f, "b"));
    ConfigurableBeanFactory::set_currently_in_creation(f, "b", true);
    assert!(ConfigurableBeanFactory::is_currently_in_creation(f, "b"));
    ConfigurableBeanFactory::set_currently_in_creation(f, "b", false);
    assert!(!ConfigurableBeanFactory::is_currently_in_creation(f, "b"));
}

#[test]
fn cbf_dependent_beans() {
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    let f = cbf(&mut c);
    ConfigurableBeanFactory::register_dependent_bean(f, "A", "B");
    assert_eq!(
        ConfigurableBeanFactory::get_dependent_beans(f, "A"),
        vec!["B"]
    );
    assert_eq!(
        ConfigurableBeanFactory::get_dependencies_for_bean(f, "B"),
        vec!["A"]
    );
}

#[test]
fn cbf_destroy_singletons() {
    let mut c = container();
    let _bean = c.get_bean_by_key(&ComponentKey::of::<String>()).unwrap();
    let f = cbf(&mut c);
    ConfigurableBeanFactory::destroy_singletons(f);
    assert!(BeanFactory::contains_bean(
        &c,
        &ComponentKey::of::<String>()
    ));
}

#[test]
fn cbf_embedded_value() {
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    let f = cbf(&mut c);
    assert_eq!(
        ConfigurableBeanFactory::resolve_embedded_value(f, "hi"),
        "hi"
    );
    ConfigurableBeanFactory::add_embedded_value_resolver(
        f,
        Arc::new(|v: &str| v.replace("${x}", "y")),
    );
    assert_eq!(
        ConfigurableBeanFactory::resolve_embedded_value(f, "${x}z"),
        "yz"
    );
}

#[test]
fn cbf_embedded_value_chain() {
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    let f = cbf(&mut c);
    ConfigurableBeanFactory::add_embedded_value_resolver(
        f,
        Arc::new(|v: &str| v.replace("${a}", "b")),
    );
    ConfigurableBeanFactory::add_embedded_value_resolver(f, Arc::new(|v: &str| v.to_uppercase()));
    assert_eq!(
        ConfigurableBeanFactory::resolve_embedded_value(f, "${a}c"),
        "BC"
    );
}

// ── BeanDefinitionRegistry remove paths ────────────────────────

#[test]
fn remove_from_registry_creates_deleted() {
    let mut c = container();
    let tn = std::any::type_name::<String>();
    assert!(BeanDefinitionRegistry::remove_bean_definition(&mut c, tn).is_ok());
    assert!(BeanDefinitionRegistry::remove_bean_definition(&mut c, tn).is_err());
}

#[test]
fn count_with_dynamic() {
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    let mut c = container();
    let before = BeanDefinitionRegistry::bean_definition_count(&c);
    let def = Box::new(RootBeanDefinition::new()) as Box<dyn vernal_beans::BeanDefinition>;
    BeanDefinitionRegistry::register_bean_definition(&mut c, "e".to_string(), def).unwrap();
    assert_eq!(
        BeanDefinitionRegistry::bean_definition_count(&c),
        before + 1
    );
}
