//! 对标 `org.springframework.beans.factory.aspectj.ConfigurableObject` 标记接口。
//!
//! 标记需要通过 AspectJ 切面进行依赖注入的对象。

/// 可配置对象标记 trait。
///
/// 对标 Spring 的 `ConfigurableObject` 接口。
/// 标记需要通过 AspectJ 切面进行依赖注入的对象。
///
/// # 用途
///
/// 实现此 trait 的类型会被 AspectJ 切面选中，在对象创建后自动进行依赖注入。
///
/// # Example
///
/// ```rust
/// use vernal_aspects::beans::factory::aspectj::ConfigurableObject;
///
/// struct MyDomainObject {
///     name: String,
/// }
///
/// impl ConfigurableObject for MyDomainObject {}
/// ```
pub trait ConfigurableObject: Send + Sync + 'static {
    /// 获取对象的配置信息。
    fn get_config_info(&self) -> Option<String> {
        None
    }
}

/// 可配置对象标记 trait（带生命周期）。
#[allow(dead_code)] // Java 镜像脚手架：带生命周期的标记 trait，供引用类型的场景使用
pub trait ConfigurableObjectWithLifetime: Send + Sync {
    /// 获取对象的配置信息。
    fn get_config_info(&self) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct TestDomainObject {
        name: String,
    }

    impl ConfigurableObject for TestDomainObject {}

    #[test]
    fn test_configurable_object_default() {
        let obj = TestDomainObject {
            name: "test".to_string(),
        };
        assert!(obj.get_config_info().is_none());
    }

    #[test]
    fn test_configurable_object_with_lifetime() {
        #[allow(dead_code)] // 测试脚手架：仅用于验证带生命周期的标记 trait
        struct TestDomainObjectWithLifetime<'a> {
            name: &'a str,
        }

        impl<'a> ConfigurableObjectWithLifetime for TestDomainObjectWithLifetime<'a> {}

        let obj = TestDomainObjectWithLifetime {
            name: "test",
        };
        assert!(obj.get_config_info().is_none());
    }

    #[test]
    fn test_configurable_object_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<TestDomainObject>();
        assert_sync::<TestDomainObject>();
    }

    #[test]
    fn test_configurable_object_trait() {
        fn assert_impl<T: ConfigurableObject>() {}
        assert_impl::<TestDomainObject>();
    }

    #[test]
    fn test_configurable_object_with_lifetime_trait() {
        fn assert_impl<T: ConfigurableObjectWithLifetime>() {}
        #[allow(dead_code)] // 测试脚手架：仅用于验证带生命周期的标记 trait
        struct Test<'a>(&'a str);
        impl<'a> ConfigurableObjectWithLifetime for Test<'a> {}
        assert_impl::<Test<'_>>();
    }

    #[test]
    fn test_configurable_object_debug() {
        let obj = TestDomainObject {
            name: "test".to_string(),
        };
        let debug_str = format!("{:?}", obj);
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_configurable_object_clone() {
        let obj = TestDomainObject {
            name: "test".to_string(),
        };
        let cloned = obj.clone();
        assert_eq!(obj.name, cloned.name);
    }

    #[test]
    fn test_configurable_object_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        let obj1 = TestDomainObject { name: "foo".to_string() };
        let obj2 = TestDomainObject { name: "bar".to_string() };
        map.insert(obj1, 1);
        map.insert(obj2, 2);
        assert_eq!(map.len(), 2);
    }
}
