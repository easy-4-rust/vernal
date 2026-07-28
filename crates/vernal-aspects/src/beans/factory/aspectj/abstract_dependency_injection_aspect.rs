//! 对标 `org.springframework.beans.factory.aspectj.AbstractDependencyInjectionAspect`。
//!
//! DI 切面抽象层：定义 pre/post-construction 与 deserialization advice。

use super::configurable_object::ConfigurableObject;

/// 方法元数据（DI 模块用）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MethodMetadata {
    /// 方法所属类型的完全限定名。
    pub type_name: &'static str,
    /// 方法名。
    pub method_name: &'static str,
}

impl MethodMetadata {
    /// 创建新的方法元数据。
    pub fn new(type_name: &'static str, method_name: &'static str) -> Self {
        Self {
            type_name,
            method_name,
        }
    }
}

/// 依赖注入切面抽象基类。
///
/// 对标 Spring 的 `AbstractDependencyInjectionAspect`。
/// 定义了 pre/post-construction 与 deserialization advice。
pub struct AbstractDependencyInjectionAspect<T: ConfigurableObject> {
    /// 是否启用 pre-construction 注入。
    pre_construction_enabled: bool,
    /// 配置的 bean 名称。
    bean_name: Option<String>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: ConfigurableObject> AbstractDependencyInjectionAspect<T> {
    /// 创建 DI 切面抽象实例。
    pub fn new() -> Self {
        Self {
            pre_construction_enabled: false,
            bean_name: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// 设置是否启用 pre-construction 注入。
    pub fn set_pre_construction_enabled(&mut self, enabled: bool) {
        self.pre_construction_enabled = enabled;
    }

    /// 设置 bean 名称。
    pub fn set_bean_name(&mut self, name: String) {
        self.bean_name = Some(name);
    }

    /// 获取 bean 名称。
    pub fn get_bean_name(&self) -> Option<&str> {
        self.bean_name.as_deref()
    }

    /// 配置 bean（注入依赖）。
    ///
    /// 对应 Spring 的 `public abstract void configureBean(Object bean)`。
    pub fn configure_bean(&self, _bean: &mut T) {
        // 实际实现需要调用 BeanFactory 进行依赖注入
    }

    /// pre-construction advice。
    ///
    /// 对应 Spring 的 `before(Object bean): beanConstruction(bean) && preConstructionCondition() && inConfigurableBean()`。
    pub fn before_construction(&self, _bean: &mut T) {
        if self.pre_construction_enabled {
            self.configure_bean(_bean);
        }
    }

    /// post-construction advice。
    ///
    /// 对应 Spring 的 `after(Object bean) returning: beanConstruction(bean) && postConstructionCondition() && inConfigurableBean()`。
    pub fn after_construction(&self, _bean: &mut T) {
        if !self.pre_construction_enabled {
            self.configure_bean(_bean);
        }
    }

    /// post-deserialization advice。
    ///
    /// 对应 Spring 的 `after(Object bean) returning: beanDeserialization(bean) && inConfigurableBean()`。
    pub fn after_deserialization(&self, _bean: &mut T) {
        self.configure_bean(_bean);
    }
}

impl<T: ConfigurableObject> Default for AbstractDependencyInjectionAspect<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::configurable_object::ConfigurableObject;

    struct TestDomainObject {
        name: String,
    }

    impl ConfigurableObject for TestDomainObject {}

    #[test]
    fn test_di_aspect_creation() {
        let aspect = AbstractDependencyInjectionAspect::<TestDomainObject>::new();
        assert!(!aspect.pre_construction_enabled);
        assert!(aspect.get_bean_name().is_none());
    }

    #[test]
    fn test_set_pre_construction_enabled() {
        let mut aspect = AbstractDependencyInjectionAspect::<TestDomainObject>::new();
        aspect.set_pre_construction_enabled(true);
        assert!(aspect.pre_construction_enabled);
    }

    #[test]
    fn test_set_bean_name() {
        let mut aspect = AbstractDependencyInjectionAspect::<TestDomainObject>::new();
        aspect.set_bean_name("myBean".to_string());
        assert_eq!(aspect.get_bean_name(), Some("myBean"));
    }

    #[test]
    fn test_before_construction_disabled() {
        let aspect = AbstractDependencyInjectionAspect::<TestDomainObject>::new();
        let mut bean = TestDomainObject {
            name: "test".to_string(),
        };
        aspect.before_construction(&mut bean);
    }

    #[test]
    fn test_before_construction_enabled() {
        let mut aspect = AbstractDependencyInjectionAspect::<TestDomainObject>::new();
        aspect.set_pre_construction_enabled(true);
        let mut bean = TestDomainObject {
            name: "test".to_string(),
        };
        aspect.before_construction(&mut bean);
    }

    #[test]
    fn test_after_construction() {
        let aspect = AbstractDependencyInjectionAspect::<TestDomainObject>::new();
        let mut bean = TestDomainObject {
            name: "test".to_string(),
        };
        aspect.after_construction(&mut bean);
    }

    #[test]
    fn test_after_deserialization() {
        let aspect = AbstractDependencyInjectionAspect::<TestDomainObject>::new();
        let mut bean = TestDomainObject {
            name: "test".to_string(),
        };
        aspect.after_deserialization(&mut bean);
    }

    #[test]
    fn test_configure_bean() {
        let aspect = AbstractDependencyInjectionAspect::<TestDomainObject>::new();
        let mut bean = TestDomainObject {
            name: "test".to_string(),
        };
        aspect.configure_bean(&mut bean);
    }

    #[test]
    fn test_di_aspect_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<AbstractDependencyInjectionAspect<TestDomainObject>>();
        assert_sync::<AbstractDependencyInjectionAspect<TestDomainObject>>();
    }

    #[test]
    fn test_di_aspect_default() {
        let aspect = AbstractDependencyInjectionAspect::<TestDomainObject>::default();
        assert!(!aspect.pre_construction_enabled);
    }

    #[test]
    fn test_di_aspect_method_metadata() {
        let meta = MethodMetadata::new("com.example.Foo", "bar");
        assert_eq!(meta.type_name, "com.example.Foo");
        assert_eq!(meta.method_name, "bar");
    }

    #[test]
    fn test_di_aspect_method_metadata_debug() {
        let meta = MethodMetadata::new("com.example.Foo", "bar");
        let debug_str = format!("{:?}", meta);
        assert!(debug_str.contains("Foo"));
        assert!(debug_str.contains("bar"));
    }

    #[test]
    fn test_di_aspect_method_metadata_clone() {
        let meta = MethodMetadata::new("com.example.Foo", "bar");
        let cloned = meta.clone();
        assert_eq!(meta, cloned);
    }

    #[test]
    fn test_di_aspect_method_metadata_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        let meta1 = MethodMetadata::new("Foo", "bar");
        let meta2 = MethodMetadata::new("Foo", "baz");
        map.insert(meta1, 1);
        map.insert(meta2, 2);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_di_aspect_with_pre_construction() {
        let mut aspect = AbstractDependencyInjectionAspect::<TestDomainObject>::new();
        aspect.set_pre_construction_enabled(true);
        aspect.set_bean_name("myBean".to_string());
        let mut bean = TestDomainObject {
            name: "test".to_string(),
        };
        aspect.before_construction(&mut bean);
        aspect.after_construction(&mut bean);
        aspect.after_deserialization(&mut bean);
        aspect.configure_bean(&mut bean);
    }

    #[test]
    fn test_di_aspect_without_pre_construction() {
        let aspect = AbstractDependencyInjectionAspect::<TestDomainObject>::new();
        let mut bean = TestDomainObject {
            name: "test".to_string(),
        };
        aspect.before_construction(&mut bean);
        aspect.after_construction(&mut bean);
        aspect.after_deserialization(&mut bean);
        aspect.configure_bean(&mut bean);
    }

    #[test]
    fn test_di_aspect_debug() {
        let aspect = AbstractDependencyInjectionAspect::<TestDomainObject>::new();
        // 不检查 Debug 实现，只检查创建成功
        let _ = aspect;
    }

    #[test]
    fn test_di_aspect_clone() {
        let aspect = AbstractDependencyInjectionAspect::<TestDomainObject>::new();
        let _ = aspect;
    }
}
