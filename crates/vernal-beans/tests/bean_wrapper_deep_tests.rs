/// BeanWrapperImpl 和 AbstractNestablePropertyAccessor 深度测试。
use std::any::Any;
use std::sync::Arc;

// ═══════════════════════════════════════════════════════════════════
// BeanWrapperImpl 深度测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bean_wrapper_new() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    assert_eq!(wrapper.property_count(), 0);
}

#[test]
fn bean_wrapper_register_multiple() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    wrapper.register_property("a", std::any::TypeId::of::<i32>());
    wrapper.register_property("b", std::any::TypeId::of::<String>());
    wrapper.register_property("c", std::any::TypeId::of::<f64>());
    assert_eq!(wrapper.property_count(), 3);
}

#[test]
fn bean_wrapper_set_overwrite() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    wrapper.register_property("name", std::any::TypeId::of::<String>());
    wrapper.set_property_value("name", Arc::new("Alice".to_string())).unwrap();
    wrapper.set_property_value("name", Arc::new("Bob".to_string())).unwrap();
    let val = wrapper.get_property_value("name").unwrap();
    assert_eq!(val.downcast_ref::<String>().unwrap(), "Bob");
}

#[test]
fn bean_wrapper_readonly_multiple() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    wrapper.register_readonly_property("ro1", std::any::TypeId::of::<i32>());
    wrapper.register_readonly_property("ro2", std::any::TypeId::of::<String>());
    assert!(wrapper.is_readable("ro1"));
    assert!(!wrapper.is_writable("ro1"));
    assert!(wrapper.is_readable("ro2"));
    assert!(!wrapper.is_writable("ro2"));
}

#[test]
fn bean_wrapper_nested_deep() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    use std::collections::HashMap;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    wrapper.register_property("user", std::any::TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>());
    let mut inner = HashMap::new();
    let mut address = HashMap::new();
    address.insert("city".to_string(), Arc::new("Beijing".to_string()) as Arc<dyn Any + Send + Sync>);
    inner.insert("address".to_string(), Arc::new(address) as Arc<dyn Any + Send + Sync>);
    wrapper.set_property_value("user", Arc::new(inner)).unwrap();
    let val = wrapper.get_property_value("user.address.city");
    assert!(val.is_ok());
    assert_eq!(val.unwrap().downcast_ref::<String>().unwrap(), "Beijing");
}

#[test]
fn bean_wrapper_nested_not_found() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    assert!(wrapper.get_property_value("nonexistent.field").is_err());
}

#[test]
fn bean_wrapper_get_property_type_nested() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    use std::collections::HashMap;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    wrapper.register_property("data", std::any::TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>());
    let mut inner = HashMap::new();
    inner.insert("value".to_string(), Arc::new(42i32) as Arc<dyn Any + Send + Sync>);
    wrapper.set_property_value("data", Arc::new(inner)).unwrap();
    let t = wrapper.get_property_type("data.value");
    assert_eq!(t, Some(std::any::TypeId::of::<i32>()));
}

#[test]
fn bean_wrapper_is_readable_writable() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    wrapper.register_property("writable", std::any::TypeId::of::<String>());
    wrapper.register_readonly_property("readonly", std::any::TypeId::of::<String>());
    assert!(wrapper.is_readable("writable"));
    assert!(wrapper.is_writable("writable"));
    assert!(wrapper.is_readable("readonly"));
    assert!(!wrapper.is_writable("readonly"));
    assert!(!wrapper.is_readable("nonexistent"));
    assert!(!wrapper.is_writable("nonexistent"));
}

#[test]
fn bean_wrapper_get_property_names_multiple() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
    wrapper.register_property("x", std::any::TypeId::of::<i32>());
    wrapper.register_property("y", std::any::TypeId::of::<String>());
    wrapper.register_property("z", std::any::TypeId::of::<f64>());
    let mut names = wrapper.get_property_names();
    names.sort();
    assert_eq!(names, vec!["x", "y", "z"]);
}

#[test]
fn bean_wrapper_batch_set() {
    use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
    use vernal_beans::property_accessor::PropertyAccessor;
    use vernal_beans::bean_wrapper::BeanWrapper;
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

// ═══════════════════════════════════════════════════════════════════
// AbstractNestablePropertyAccessor 深度测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn nestable_new() {
    use vernal_beans::abstract_nestable_property_accessor::AbstractNestablePropertyAccessor;
    let accessor = AbstractNestablePropertyAccessor::new();
    assert_eq!(accessor.property_count(), 0);
}

#[test]
fn nestable_register_property() {
    use vernal_beans::abstract_nestable_property_accessor::AbstractNestablePropertyAccessor;
    use vernal_beans::property_accessor::PropertyAccessor;
    let accessor = AbstractNestablePropertyAccessor::new();
    accessor.register_property("name", std::any::TypeId::of::<String>());
    assert!(accessor.has_property("name"));
    assert!(accessor.is_readable("name"));
    assert!(accessor.is_writable("name"));
}

#[test]
fn nestable_register_readonly() {
    use vernal_beans::abstract_nestable_property_accessor::AbstractNestablePropertyAccessor;
    use vernal_beans::property_accessor::PropertyAccessor;
    let accessor = AbstractNestablePropertyAccessor::new();
    accessor.register_readonly_property("ro", std::any::TypeId::of::<i32>());
    assert!(accessor.is_readable("ro"));
    assert!(!accessor.is_writable("ro"));
}

#[test]
fn nestable_set_get_property() {
    use vernal_beans::abstract_nestable_property_accessor::AbstractNestablePropertyAccessor;
    use vernal_beans::property_accessor::PropertyAccessor;
    let accessor = AbstractNestablePropertyAccessor::new();
    accessor.register_property("name", std::any::TypeId::of::<String>());
    accessor.set_property_value("name", Arc::new("Alice".to_string())).unwrap();
    let val = accessor.get_property_value("name").unwrap();
    assert_eq!(val.downcast_ref::<String>().unwrap(), "Alice");
}

#[test]
fn nestable_nested_property_two_levels() {
    use vernal_beans::abstract_nestable_property_accessor::AbstractNestablePropertyAccessor;
    use vernal_beans::property_accessor::PropertyAccessor;
    use std::collections::HashMap;
    let accessor = AbstractNestablePropertyAccessor::new();
    accessor.register_property("address", std::any::TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>());
    let mut inner = HashMap::new();
    inner.insert("city".to_string(), Arc::new("Beijing".to_string()) as Arc<dyn Any + Send + Sync>);
    accessor.set_property_value("address", Arc::new(inner)).unwrap();
    let val = accessor.get_property_value("address.city");
    assert!(val.is_ok());
    assert_eq!(val.unwrap().downcast_ref::<String>().unwrap(), "Beijing");
}

#[test]
fn nestable_nested_property_three_levels() {
    use vernal_beans::abstract_nestable_property_accessor::AbstractNestablePropertyAccessor;
    use vernal_beans::property_accessor::PropertyAccessor;
    use std::collections::HashMap;
    let accessor = AbstractNestablePropertyAccessor::new();
    accessor.register_property("user", std::any::TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>());
    let mut inner = HashMap::new();
    let mut address = HashMap::new();
    address.insert("city".to_string(), Arc::new("Beijing".to_string()) as Arc<dyn Any + Send + Sync>);
    inner.insert("address".to_string(), Arc::new(address) as Arc<dyn Any + Send + Sync>);
    accessor.set_property_value("user", Arc::new(inner)).unwrap();
    let val = accessor.get_property_value("user.address.city");
    assert!(val.is_ok());
    assert_eq!(val.unwrap().downcast_ref::<String>().unwrap(), "Beijing");
}

#[test]
fn nestable_nested_not_found() {
    use vernal_beans::abstract_nestable_property_accessor::AbstractNestablePropertyAccessor;
    use vernal_beans::property_accessor::PropertyAccessor;
    let accessor = AbstractNestablePropertyAccessor::new();
    assert!(accessor.get_property_value("nonexistent.field").is_err());
}

#[test]
fn nestable_property_type() {
    use vernal_beans::abstract_nestable_property_accessor::AbstractNestablePropertyAccessor;
    use vernal_beans::property_accessor::PropertyAccessor;
    let accessor = AbstractNestablePropertyAccessor::new();
    accessor.register_property("name", std::any::TypeId::of::<String>());
    assert_eq!(accessor.get_property_type("name"), Some(std::any::TypeId::of::<String>()));
    assert_eq!(accessor.get_property_type("missing"), None);
}

#[test]
fn nestable_property_names() {
    use vernal_beans::abstract_nestable_property_accessor::AbstractNestablePropertyAccessor;
    use vernal_beans::property_accessor::PropertyAccessor;
    let accessor = AbstractNestablePropertyAccessor::new();
    accessor.register_property("a", std::any::TypeId::of::<i32>());
    accessor.register_property("b", std::any::TypeId::of::<String>());
    let mut names = accessor.get_property_names();
    names.sort();
    assert_eq!(names, vec!["a", "b"]);
}

#[test]
fn nestable_has_property() {
    use vernal_beans::abstract_nestable_property_accessor::AbstractNestablePropertyAccessor;
    let accessor = AbstractNestablePropertyAccessor::new();
    accessor.register_property("exists", std::any::TypeId::of::<String>());
    assert!(accessor.has_property("exists"));
    assert!(!accessor.has_property("nonexistent"));
}

#[test]
fn nestable_property_count() {
    use vernal_beans::abstract_nestable_property_accessor::AbstractNestablePropertyAccessor;
    let accessor = AbstractNestablePropertyAccessor::new();
    assert_eq!(accessor.property_count(), 0);
    accessor.register_property("a", std::any::TypeId::of::<i32>());
    assert_eq!(accessor.property_count(), 1);
}

#[test]
fn nestable_set_overwrite() {
    use vernal_beans::abstract_nestable_property_accessor::AbstractNestablePropertyAccessor;
    use vernal_beans::property_accessor::PropertyAccessor;
    let accessor = AbstractNestablePropertyAccessor::new();
    accessor.register_property("name", std::any::TypeId::of::<String>());
    accessor.set_property_value("name", Arc::new("Alice".to_string())).unwrap();
    accessor.set_property_value("name", Arc::new("Bob".to_string())).unwrap();
    let val = accessor.get_property_value("name").unwrap();
    assert_eq!(val.downcast_ref::<String>().unwrap(), "Bob");
}

#[test]
fn nestable_is_readable_writable() {
    use vernal_beans::abstract_nestable_property_accessor::AbstractNestablePropertyAccessor;
    use vernal_beans::property_accessor::PropertyAccessor;
    let accessor = AbstractNestablePropertyAccessor::new();
    accessor.register_property("writable", std::any::TypeId::of::<String>());
    accessor.register_readonly_property("readonly", std::any::TypeId::of::<String>());
    assert!(accessor.is_readable("writable"));
    assert!(accessor.is_writable("writable"));
    assert!(accessor.is_readable("readonly"));
    assert!(!accessor.is_writable("readonly"));
    assert!(!accessor.is_readable("nonexistent"));
    assert!(!accessor.is_writable("nonexistent"));
}
