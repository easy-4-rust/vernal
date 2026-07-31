//! 对标 `org.springframework.beans.factory.aspectj.ConfigurableObject` 标记接口。
//!
//! 对应 Java：`ConfigurableObject.java`（`spring-aspects` 模块）
//! 包路径：`org.springframework.beans.factory.aspectj`
//! 核心职责：标记需要通过 AspectJ 切面进行依赖注入的对象。
//!
//! 注意：按审计脚本"保留末两层"规则，本文件位于 `factory/aspectj/` 而非 `beans/factory/aspectj/`。

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
/// use vernal_aspects::factory::aspectj::ConfigurableObject;
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
