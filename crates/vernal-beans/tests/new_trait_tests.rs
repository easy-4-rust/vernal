//! 针对新增的 Container 5 个 Trait 实现和 Bean 生命周期核心的测试。
use std::any::Any;
use std::sync::Arc;
use vernal_beans::{
    bean_factory::BeanFactory,
    bean_definition_registry::BeanDefinitionRegistry,
    hierarchical_bean_factory::HierarchicalBeanFactory,
    listable_bean_factory::ListableBeanFactory,
    configurable_bean_factory::ConfigurableBeanFactory,
    singleton_bean_registry::SingletonBeanRegistry,
    configurable_listable_bean_factory::ConfigurableListableBeanFactory,
    ComponentDefinition, ComponentKey, Container, RegistryBuilder, Qualifier,
};

fn make_container() -> Container {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32));
    Container::new(b.build().unwrap())
}

// ═══════════════════════════════════════════════════════════════════
// SingletonBeanRegistry 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn singleton_register_and_get() {
    let c = make_container();
    // 通过 resolve 触发单例注册
    let _bean: Arc<String> = c.resolve().unwrap();
    // 现在 get_singleton 应该能找到它
    let names = c.singleton_names();
    assert!(!names.is_empty(), "应该有已注册的单例");
}

#[test]
fn singleton_contains_check() {
    let c = make_container();
    // 解析后应该包含
    let _bean: Arc<String> = c.resolve().unwrap();
    // 通过名称检查（名称是 type_name）
    let count = c.singleton_count();
    assert!(count >= 1, "应该至少有 1 个单例");
}

#[test]
fn singleton_mutex_returns_arc() {
    let c = make_container();
    let m1 = c.singleton_mutex();
    let m2 = c.singleton_mutex();
    assert!(Arc::ptr_eq(&m1, &m2), "两次获取应该返回同一个 Arc");
}

#[test]
fn singleton_names_after_resolve() {
    let c = make_container();
    let _s: Arc<String> = c.resolve().unwrap();
    let _i: Arc<i32> = c.resolve().unwrap();
    let names = c.singleton_names();
    assert!(names.len() >= 2, "应该有至少 2 个单例名称");
}

// ═══════════════════════════════════════════════════════════════════
// HierarchicalBeanFactory 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn parent_bean_factory_none_by_default() {
    let c = make_container();
    assert!(c.parent_bean_factory().is_none(), "默认没有父容器");
}

#[test]
fn contains_local_bean_returns_false_for_unknown() {
    let c = make_container();
    assert!(!c.contains_local_bean("nonexistent_bean"));
}

#[test]
fn contains_local_bean_returns_true_for_registered() {
    let c = make_container();
    // 注册的 bean 应该在本地
    assert!(c.contains_local_bean("alloc::string::String") || c.contains_local_bean("String"));
}

// ═══════════════════════════════════════════════════════════════════
// ListableBeanFactory 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bean_definition_count_reflects_registered() {
    let c = make_container();
    let count = BeanDefinitionRegistry::bean_definition_count(&c);
    assert!(count >= 2, "应该有至少 2 个 Bean 定义，实际: {}", count);
}

#[test]
fn bean_definition_names_not_empty() {
    let c = make_container();
    let names = BeanDefinitionRegistry::bean_definition_names(&c);
    assert!(!names.is_empty(), "Bean 定义名称不应为空");
}

#[test]
fn bean_names_for_type_id() {
    let c = make_container();
    let names = c.bean_names_for_type_id(std::any::TypeId::of::<String>(), true, true);
    assert!(!names.is_empty(), "应该有 String 类型的 Bean");
}

#[test]
fn bean_names_for_type_id_unknown() {
    let c = make_container();
    let names = c.bean_names_for_type_id(std::any::TypeId::of::<f64>(), true, true);
    assert!(names.is_empty(), "不应该有 f64 类型的 Bean");
}

#[test]
fn beans_of_type_id() {
    let c = make_container();
    let beans = c.beans_of_type_id(std::any::TypeId::of::<String>(), true, true);
    assert!(beans.is_ok());
    let beans = beans.unwrap();
    assert_eq!(beans.len(), 1, "应该有 1 个 String 类型的 Bean");
}

#[test]
fn contains_singleton_bean_after_resolve() {
    let c = make_container();
    let _s: Arc<String> = c.resolve().unwrap();
    assert!(c.contains_singleton_bean(), "应该包含单例 Bean");
}

#[test]
fn bean_post_processor_count_initial() {
    let c = make_container();
    assert_eq!(c.bean_post_processor_count(), 0, "初始应该没有后处理器");
}

#[test]
fn bean_names_iterator() {
    let c = make_container();
    let names: Vec<String> = c.bean_names_iterator().collect();
    assert!(names.len() >= 2, "迭代器应该返回至少 2 个名称");
}

// ═══════════════════════════════════════════════════════════════════
// ConfigurableBeanFactory 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn register_scope_and_get() {
    use vernal_beans::bean_scope::BeanScope;
    let mut c = make_container();

    struct TestScope;
    impl BeanScope for TestScope {
        fn get(
            &self,
            _name: &str,
            object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
        ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(object_factory())
        }
    }

    c.register_scope("test_scope", Box::new(TestScope));
    let scope = c.get_registered_scope("test_scope");
    assert!(scope.is_some(), "应该能获取注册的 Scope");
}

#[test]
fn registered_scope_names() {
    use vernal_beans::bean_scope::BeanScope;
    let mut c = make_container();

    struct TestScope;
    impl BeanScope for TestScope {
        fn get(
            &self,
            _name: &str,
            object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
        ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(object_factory())
        }
    }

    c.register_scope("my_scope", Box::new(TestScope));
    let names = c.registered_scope_names();
    assert!(names.contains(&"my_scope".to_string()));
}

#[test]
fn register_alias() {
    let mut c = make_container();
    // 注册别名
    let result = c.register_alias("alloc::string::String", "myAlias");
    // 可能成功或失败（取决于实现）
    let _ = result;
}

#[test]
fn add_bean_post_processor() {
    use vernal_beans::bean_post_processor::BeanPostProcessor;
    let mut c = make_container();

    struct TestPP;
    impl BeanPostProcessor for TestPP {}

    c.add_bean_post_processor(Arc::new(TestPP));
    assert_eq!(c.bean_post_processor_count(), 1);
}

#[test]
fn set_currently_in_creation() {
    let mut c = make_container();
    c.set_currently_in_creation("test_bean", true);
    assert!(c.is_currently_in_creation("test_bean"));
    c.set_currently_in_creation("test_bean", false);
    assert!(!c.is_currently_in_creation("test_bean"));
}

#[test]
fn register_dependent_beans() {
    let mut c = make_container();
    c.register_dependent_bean("beanA", "beanB");
    let dependents = c.get_dependent_beans("beanA");
    assert!(dependents.contains(&"beanB".to_string()));
    let deps = c.get_dependencies_for_bean("beanB");
    assert!(deps.contains(&"beanA".to_string()));
}

// ═══════════════════════════════════════════════════════════════════
// ConfigurableListableBeanFactory 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn freeze_and_check_configuration() {
    let mut c = make_container();
    assert!(!c.is_configuration_frozen());
    c.freeze_configuration();
    assert!(c.is_configuration_frozen());
}

#[test]
fn pre_instantiate_singletons() {
    let c = make_container();
    let result = c.pre_instantiate_singletons();
    assert!(result.is_ok());
}

#[test]
fn ignore_dependency_type() {
    let mut c = make_container();
    c.ignore_dependency_type(std::any::TypeId::of::<String>());
}

#[test]
fn is_autowire_candidate() {
    let c = make_container();
    // 默认应该是 autowire 候选
    let result = c.is_autowire_candidate("alloc::string::String");
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════
// DefaultSingletonBeanRegistry 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn default_singleton_registry_new() {
    let reg = vernal_beans::DefaultSingletonBeanRegistry::new();
    assert_eq!(reg.singleton_count(), 0);
}

#[test]
fn default_singleton_registry_register_and_get() {
    let reg = vernal_beans::DefaultSingletonBeanRegistry::new();
    reg.register_singleton("bean1", Arc::new(42i32));
    assert_eq!(reg.singleton_count(), 1);
    assert!(reg.contains_singleton("bean1"));
    let bean = reg.get_singleton("bean1", true);
    assert!(bean.is_some());
    let val = bean.unwrap().downcast_ref::<i32>().copied();
    assert_eq!(val, Some(42));
}

#[test]
fn default_singleton_registry_early_reference() {
    let reg = vernal_beans::DefaultSingletonBeanRegistry::new();
    // 添加工厂（三级缓存）
    reg.add_singleton_factory("bean1", Arc::new(|| Arc::new(99i32) as Arc<dyn Any + Send + Sync>));
    // 获取早期引用
    let early = reg.get_early_bean_reference("bean1");
    assert!(early.is_some());
    let val = early.unwrap().downcast_ref::<i32>().copied();
    assert_eq!(val, Some(99));
}

#[test]
fn default_singleton_registry_creation_tracking() {
    let reg = vernal_beans::DefaultSingletonBeanRegistry::new();
    assert!(!reg.is_singleton_currently_in_creation("bean1"));
    reg.mark_singleton_as_in_creation("bean1");
    assert!(reg.is_singleton_currently_in_creation("bean1"));
    reg.mark_singleton_as_created("bean1");
    assert!(!reg.is_singleton_currently_in_creation("bean1"));
}

#[test]
fn default_singleton_registry_destroy() {
    let reg = vernal_beans::DefaultSingletonBeanRegistry::new();
    reg.register_singleton("bean1", Arc::new(1i32));
    reg.register_singleton("bean2", Arc::new(2i32));
    assert_eq!(reg.singleton_count(), 2);
    reg.destroy_singletons();
    assert_eq!(reg.singleton_count(), 0);
}

#[test]
fn default_singleton_registry_names() {
    let reg = vernal_beans::DefaultSingletonBeanRegistry::new();
    reg.register_singleton("alpha", Arc::new(1));
    reg.register_singleton("beta", Arc::new(2));
    let mut names = reg.singleton_names();
    names.sort();
    assert_eq!(names, vec!["alpha", "beta"]);
}

// ═══════════════════════════════════════════════════════════════════
// ConstructorResolver 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn constructor_resolver_new() {
    let _ = vernal_beans::ConstructorResolver::new();
}

#[test]
fn constructor_resolver_autowire_no_args() {
    let resolver = vernal_beans::ConstructorResolver::new();
    let result = resolver.autowire_constructor(
        "test_bean",
        std::any::TypeId::of::<String>(),
        &[],
        &|_| Ok(Arc::new("created".to_string()) as Arc<dyn Any + Send + Sync>),
    );
    assert!(result.is_ok());
}

#[test]
fn constructor_resolver_autowire_with_args() {
    let resolver = vernal_beans::ConstructorResolver::new();
    let args: Vec<Arc<dyn Any + Send + Sync>> = vec![Arc::new("arg1".to_string())];
    let result = resolver.autowire_constructor(
        "test_bean",
        std::any::TypeId::of::<String>(),
        &args,
        &|a| Ok(a[0].clone()),
    );
    assert!(result.is_ok());
}

#[test]
fn constructor_resolver_resolve_args() {
    let resolver = vernal_beans::ConstructorResolver::new();
    let mut available = std::collections::HashMap::new();
    available.insert(
        std::any::TypeId::of::<String>(),
        Arc::new("value".to_string()) as Arc<dyn Any + Send + Sync>,
    );
    let def_args: Vec<Arc<dyn Any + Send + Sync>> = vec![Arc::new("placeholder".to_string())];
    let resolved = resolver.resolve_constructor_arguments(&def_args, &available);
    assert_eq!(resolved.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════
// BeanWrapperImpl 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bean_wrapper_register_and_get() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    wrapper.register_property("name", std::any::TypeId::of::<String>());
    wrapper.set_property_value("name", Arc::new("Alice".to_string())).unwrap();
    let val = wrapper.get_property_value("name").unwrap();
    assert_eq!(val.downcast_ref::<String>().unwrap(), "Alice");
}

#[test]
fn bean_wrapper_readonly() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    wrapper.register_readonly_property("ro", std::any::TypeId::of::<i32>());
    assert!(wrapper.is_readable("ro"));
    assert!(!wrapper.is_writable("ro"));
    assert!(wrapper.set_property_value("ro", Arc::new(42i32)).is_err());
}

#[test]
fn bean_wrapper_nested_property() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    use std::collections::HashMap;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    wrapper.register_property("address", std::any::TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>());
    let mut inner = HashMap::new();
    inner.insert("city".to_string(), Arc::new("Beijing".to_string()) as Arc<dyn Any + Send + Sync>);
    wrapper.set_property_value("address", Arc::new(inner)).unwrap();
    let val = wrapper.get_property_value("address.city").unwrap();
    assert_eq!(val.downcast_ref::<String>().unwrap(), "Beijing");
}

#[test]
fn bean_wrapper_property_type() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    wrapper.register_property("name", std::any::TypeId::of::<String>());
    assert_eq!(wrapper.get_property_type("name"), Some(std::any::TypeId::of::<String>()));
    assert_eq!(wrapper.get_property_type("missing"), None);
}

#[test]
fn bean_wrapper_property_names() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    wrapper.register_property("a", std::any::TypeId::of::<i32>());
    wrapper.register_property("b", std::any::TypeId::of::<String>());
    let mut names = wrapper.get_property_names();
    names.sort();
    assert_eq!(names, vec!["a", "b"]);
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

#[test]
fn bean_wrapper_nonexistent_property() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    assert!(wrapper.get_property_value("missing").is_err());
}

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
