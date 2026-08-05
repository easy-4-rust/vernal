//! BeanDefinition — Spring 风格的 Bean 元数据 trait。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.BeanDefinition`。
//!
//! 描述一个 Bean 实例的元数据，包括类型信息、作用域、依赖、初始化方法等。
//! 这是 vernal-beans 与 Spring IoC 容器语义对齐的核心接口。

use std::any::Any;
use std::fmt;

use crate::component_key::ComponentKey;
use crate::component_scope::Scope;

/// Bean 定义 trait — Spring 风格的 IoC 注册入口。
///
/// 对应 Spring 的 `BeanDefinition`；每条 `RegistryBuilder::register()` 都
/// 会把一个实现该 trait 的类型转换为不可变 `ComponentDefinition`。
///
/// ## 常量（对标 Spring）
///
/// - `SCOPE_SINGLETON = "singleton"` — 单例作用域
/// - `SCOPE_PROTOTYPE = "prototype"` — 原型（Transient）作用域
/// - `ROLE_APPLICATION = 0` — 应用 Bean（用户自定义）
/// - `ROLE_SUPPORT = 1` — 支持 Bean（配置辅助）
/// - `ROLE_INFRASTRUCTURE = 2` — 基础设施 Bean（框架内部）
///
/// ## 示例
///
/// ```rust,ignore
/// use vernal_beans::bean_definition::BeanDefinition;
///
/// struct MyService;
/// impl BeanDefinition for MyService {
///     fn bean_class_name(&self) -> &str { "MyService" }
///     fn scope(&self) -> Scope { Scope::Singleton }
///     fn is_lazy_init(&self) -> bool { false }
///     fn is_primary(&self) -> bool { false }
///     fn role(&self) -> i32 { BeanDefinition::ROLE_APPLICATION }
/// }
/// ```
pub trait BeanDefinition: Send + Sync + Any + fmt::Debug {
    /// Bean 名称（唯一标识的一部分）。
    ///
    /// 对应 Spring 的 `getBeanName()` + `getAliases()` 合并后的主名称。
    fn bean_name(&self) -> &ComponentKey;

    /// Bean 类型名（Rust 的 `std::any::type_name::<T>()`）。
    ///
    /// 对应 Spring 的 `getBeanClassName()`。
    fn bean_class_name(&self) -> &str;

    /// 作用域。
    ///
    /// 对应 Spring 的 `getScope()` 返回 `SCOPE_SINGLETON` / `SCOPE_PROTOTYPE`。
    fn scope(&self) -> Scope;

    /// 是否惰性初始化。
    ///
    /// 对应 Spring 的 `isLazyInit()`。
    /// 单例 Bean 默认 `false`（Eager），原型 Bean 始终 Lazy。
    fn is_lazy_init(&self) -> bool;

    /// 是否 primary。
    ///
    /// 对应 Spring 的 `isPrimary()`。
    /// 多个同类型候选时，primary 作为 tie-breaker。
    fn is_primary(&self) -> bool;

    /// 是否 fallback。
    ///
    /// 对应 Spring 6.2+ 的 `isFallback()`。
    /// 所有 Bean 中只有一个不是 fallback 时，该 Bean 被选中。
    fn is_fallback(&self) -> bool {
        false
    }

    /// 是否 autowire candidate。
    ///
    /// 对应 Spring 的 `isAutowireCandidate()`。
    /// `false` 时该 Bean 不参与按类型自动装配。
    fn is_autowire_candidate(&self) -> bool {
        true
    }

    /// Bean 角色（ROLE_APPLICATION / ROLE_SUPPORT / ROLE_INFRASTRUCTURE）。
    ///
    /// 对应 Spring 的 `getRole()`。
    fn role(&self) -> i32 {
        ROLE_APPLICATION
    }

    /// 描述信息。
    ///
    /// 对应 Spring 的 `getDescription()`。
    fn description(&self) -> Option<&str> {
        None
    }

    /// Bean 类名（非实例类型，用于 BeanFactory 查找）。
    ///
    /// 对应 Spring 的 `getBeanClassName()`（与 bean_class_name 相同语义）。
    fn bean_class_name_internal(&self) -> Option<&str> {
        Some(self.bean_class_name())
    }

    /// 父 Bean 定义名称。
    ///
    /// 对应 Spring 的 `getParentName()`。
    fn parent_name(&self) -> Option<&str> {
        None
    }

    /// 工厂 Bean 名称（通过工厂方法创建 Bean）。
    ///
    /// 对应 Spring 的 `getFactoryBeanName()`。
    fn factory_bean_name(&self) -> Option<&str> {
        None
    }

    /// 工厂方法名称。
    ///
    /// 对应 Spring 的 `getFactoryMethodName()`。
    fn factory_method_name(&self) -> Option<&str> {
        None
    }

    /// 初始化方法名称。
    ///
    /// 对应 Spring 的 `getInitMethodName()`。
    fn init_method_name(&self) -> Option<&str> {
        None
    }

    /// 销毁方法名称。
    ///
    /// 对应 Spring 的 `getDestroyMethodName()`。
    fn destroy_method_name(&self) -> Option<&str> {
        None
    }

    /// 是否是抽象的（不能直接 getBean）。
    ///
    /// 对应 Spring 的 `isAbstract()`。
    fn is_abstract(&self) -> bool {
        false
    }

    /// 是否是 singleton。
    ///
    /// 对应 Spring 的 `isSingleton()`。
    fn is_singleton(&self) -> bool {
        matches!(self.scope(), Scope::Singleton)
    }

    /// 是否是 prototype（Transient）。
    ///
    /// 对应 Spring 的 `isPrototype()`。
    fn is_prototype(&self) -> bool {
        matches!(self.scope(), Scope::Transient)
    }

    /// 资源描述（用于错误报告）。
    ///
    /// 对应 Spring 的 `getResourceDescription()`。
    fn resource_description(&self) -> Option<&str> {
        None
    }

    /// 返回原始的 BeanDefinition（如果被代理包装了）。
    ///
    /// 对应 Spring 的 `getOriginatingBeanDefinition()`。
    fn originating_bean_definition(&self) -> Option<&dyn BeanDefinition> {
        None
    }
}

/// 标准 singleton 作用域名称：`"singleton"`。
pub const SCOPE_SINGLETON: &str = "singleton";

/// 标准 prototype 作用域名称：`"prototype"`。
pub const SCOPE_PROTOTYPE: &str = "prototype";

/// 角色：应用 Bean（用户自定义）。
pub const ROLE_APPLICATION: i32 = 0;

/// 角色：支持 Bean（配置辅助）。
pub const ROLE_SUPPORT: i32 = 1;

/// 角色：基础设施 Bean（框架内部）。
pub const ROLE_INFRASTRUCTURE: i32 = 2;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component_key::ComponentKey;

    /// A minimal BeanDefinition implementation for testing default methods.
    #[derive(Debug)]
    struct TestBeanDefinition {
        class_name: String,
        scope_val: Scope,
        lazy: bool,
        primary: bool,
    }

    impl TestBeanDefinition {
        fn new(class_name: &str) -> Self {
            Self {
                class_name: class_name.to_string(),
                scope_val: Scope::Singleton,
                lazy: false,
                primary: false,
            }
        }

        fn with_scope(mut self, scope: Scope) -> Self {
            self.scope_val = scope;
            self
        }

        fn with_lazy(mut self, lazy: bool) -> Self {
            self.lazy = lazy;
            self
        }

        fn with_primary(mut self, primary: bool) -> Self {
            self.primary = primary;
            self
        }
    }

    impl BeanDefinition for TestBeanDefinition {
        fn bean_name(&self) -> &ComponentKey {
            unimplemented!("test stub")
        }
        fn bean_class_name(&self) -> &str {
            &self.class_name
        }
        fn scope(&self) -> Scope {
            self.scope_val
        }
        fn is_lazy_init(&self) -> bool {
            self.lazy
        }
        fn is_primary(&self) -> bool {
            self.primary
        }
    }

    #[test]
    fn default_is_fallback() {
        let bd = TestBeanDefinition::new("Test");
        assert!(!bd.is_fallback());
    }

    #[test]
    fn default_is_autowire_candidate() {
        let bd = TestBeanDefinition::new("Test");
        assert!(bd.is_autowire_candidate());
    }

    #[test]
    fn default_role_is_application() {
        let bd = TestBeanDefinition::new("Test");
        assert_eq!(bd.role(), ROLE_APPLICATION);
    }

    #[test]
    fn default_description_is_none() {
        let bd = TestBeanDefinition::new("Test");
        assert!(bd.description().is_none());
    }

    #[test]
    fn default_bean_class_name_internal() {
        let bd = TestBeanDefinition::new("MyService");
        assert_eq!(bd.bean_class_name_internal(), Some("MyService"));
    }

    #[test]
    fn default_parent_name_is_none() {
        let bd = TestBeanDefinition::new("Test");
        assert!(bd.parent_name().is_none());
    }

    #[test]
    fn default_factory_bean_name_is_none() {
        let bd = TestBeanDefinition::new("Test");
        assert!(bd.factory_bean_name().is_none());
    }

    #[test]
    fn default_factory_method_name_is_none() {
        let bd = TestBeanDefinition::new("Test");
        assert!(bd.factory_method_name().is_none());
    }

    #[test]
    fn default_init_method_name_is_none() {
        let bd = TestBeanDefinition::new("Test");
        assert!(bd.init_method_name().is_none());
    }

    #[test]
    fn default_destroy_method_name_is_none() {
        let bd = TestBeanDefinition::new("Test");
        assert!(bd.destroy_method_name().is_none());
    }

    #[test]
    fn default_is_abstract() {
        let bd = TestBeanDefinition::new("Test");
        assert!(!bd.is_abstract());
    }

    #[test]
    fn is_singleton_for_singleton_scope() {
        let bd = TestBeanDefinition::new("Test").with_scope(Scope::Singleton);
        assert!(bd.is_singleton());
        assert!(!bd.is_prototype());
    }

    #[test]
    fn is_prototype_for_transient_scope() {
        let bd = TestBeanDefinition::new("Test").with_scope(Scope::Transient);
        assert!(!bd.is_singleton());
        assert!(bd.is_prototype());
    }

    #[test]
    fn default_resource_description_is_none() {
        let bd = TestBeanDefinition::new("Test");
        assert!(bd.resource_description().is_none());
    }

    #[test]
    fn default_originating_bean_definition_is_none() {
        let bd = TestBeanDefinition::new("Test");
        assert!(bd.originating_bean_definition().is_none());
    }

    #[test]
    fn constants_are_correct() {
        assert_eq!(SCOPE_SINGLETON, "singleton");
        assert_eq!(SCOPE_PROTOTYPE, "prototype");
        assert_eq!(ROLE_APPLICATION, 0);
        assert_eq!(ROLE_SUPPORT, 1);
        assert_eq!(ROLE_INFRASTRUCTURE, 2);
    }

    #[test]
    fn test_bean_class_name() {
        let bd = TestBeanDefinition::new("MyService");
        assert_eq!(bd.bean_class_name(), "MyService");
    }

    #[test]
    fn test_scope_singleton() {
        let bd = TestBeanDefinition::new("Test").with_scope(Scope::Singleton);
        assert_eq!(bd.scope(), Scope::Singleton);
    }

    #[test]
    fn test_scope_transient() {
        let bd = TestBeanDefinition::new("Test").with_scope(Scope::Transient);
        assert_eq!(bd.scope(), Scope::Transient);
    }

    #[test]
    fn test_is_lazy_init_true() {
        let bd = TestBeanDefinition::new("Test").with_lazy(true);
        assert!(bd.is_lazy_init());
    }

    #[test]
    fn test_is_lazy_init_false() {
        let bd = TestBeanDefinition::new("Test").with_lazy(false);
        assert!(!bd.is_lazy_init());
    }

    #[test]
    fn test_is_primary_true() {
        let bd = TestBeanDefinition::new("Test").with_primary(true);
        assert!(bd.is_primary());
    }

    #[test]
    fn test_is_primary_false() {
        let bd = TestBeanDefinition::new("Test").with_primary(false);
        assert!(!bd.is_primary());
    }

    #[test]
    fn test_is_singleton_and_not_prototype() {
        let bd = TestBeanDefinition::new("Test").with_scope(Scope::Singleton);
        assert!(bd.is_singleton());
        assert!(!bd.is_prototype());
    }

    #[test]
    fn test_is_prototype_and_not_singleton() {
        let bd = TestBeanDefinition::new("Test").with_scope(Scope::Transient);
        assert!(bd.is_prototype());
        assert!(!bd.is_singleton());
    }

    #[test]
    fn test_default_methods_return_none() {
        let bd = TestBeanDefinition::new("Test");
        assert!(bd.is_fallback() == false);
        assert!(bd.is_autowire_candidate());
        assert_eq!(bd.role(), ROLE_APPLICATION);
        assert!(bd.description().is_none());
        assert!(bd.bean_class_name_internal().is_some());
        assert!(bd.parent_name().is_none());
        assert!(bd.factory_bean_name().is_none());
        assert!(bd.factory_method_name().is_none());
        assert!(bd.init_method_name().is_none());
        assert!(bd.destroy_method_name().is_none());
        assert!(!bd.is_abstract());
        assert!(bd.resource_description().is_none());
        assert!(bd.originating_bean_definition().is_none());
    }

    #[test]
    fn test_bean_class_name_internal_returns_class_name() {
        let bd = TestBeanDefinition::new("com.example.MyService");
        assert_eq!(bd.bean_class_name_internal(), Some("com.example.MyService"));
    }

    #[test]
    fn test_role_support() {
        #[derive(Debug)]
        struct SupportBean;
        impl BeanDefinition for SupportBean {
            fn bean_name(&self) -> &ComponentKey {
                unimplemented!()
            }
            fn bean_class_name(&self) -> &str {
                "SupportBean"
            }
            fn scope(&self) -> Scope {
                Scope::Singleton
            }
            fn is_lazy_init(&self) -> bool {
                false
            }
            fn is_primary(&self) -> bool {
                false
            }
            fn role(&self) -> i32 {
                ROLE_SUPPORT
            }
        }
        let bd = SupportBean;
        assert_eq!(bd.role(), ROLE_SUPPORT);
    }

    #[test]
    fn test_role_infrastructure() {
        #[derive(Debug)]
        struct InfraBean;
        impl BeanDefinition for InfraBean {
            fn bean_name(&self) -> &ComponentKey {
                unimplemented!()
            }
            fn bean_class_name(&self) -> &str {
                "InfraBean"
            }
            fn scope(&self) -> Scope {
                Scope::Singleton
            }
            fn is_lazy_init(&self) -> bool {
                false
            }
            fn is_primary(&self) -> bool {
                false
            }
            fn role(&self) -> i32 {
                ROLE_INFRASTRUCTURE
            }
        }
        let bd = InfraBean;
        assert_eq!(bd.role(), ROLE_INFRASTRUCTURE);
    }

    // ── Additional coverage tests ──────────────────────────────────────────

    #[test]
    fn test_is_fallback_default() {
        let bd = TestBeanDefinition::new("Test");
        assert!(!bd.is_fallback());
    }

    #[test]
    fn test_is_autowire_candidate_default() {
        let bd = TestBeanDefinition::new("Test");
        assert!(bd.is_autowire_candidate());
    }

    #[test]
    fn test_description_default() {
        let bd = TestBeanDefinition::new("Test");
        assert!(bd.description().is_none());
    }

    #[test]
    fn test_parent_name_default() {
        let bd = TestBeanDefinition::new("Test");
        assert!(bd.parent_name().is_none());
    }

    #[test]
    fn test_factory_bean_name_default() {
        let bd = TestBeanDefinition::new("Test");
        assert!(bd.factory_bean_name().is_none());
    }

    #[test]
    fn test_factory_method_name_default() {
        let bd = TestBeanDefinition::new("Test");
        assert!(bd.factory_method_name().is_none());
    }

    #[test]
    fn test_init_method_name_default() {
        let bd = TestBeanDefinition::new("Test");
        assert!(bd.init_method_name().is_none());
    }

    #[test]
    fn test_destroy_method_name_default() {
        let bd = TestBeanDefinition::new("Test");
        assert!(bd.destroy_method_name().is_none());
    }

    #[test]
    fn test_is_abstract_default() {
        let bd = TestBeanDefinition::new("Test");
        assert!(!bd.is_abstract());
    }

    #[test]
    fn test_resource_description_default() {
        let bd = TestBeanDefinition::new("Test");
        assert!(bd.resource_description().is_none());
    }

    #[test]
    fn test_originating_bean_definition_default() {
        let bd = TestBeanDefinition::new("Test");
        assert!(bd.originating_bean_definition().is_none());
    }

    #[test]
    fn test_constants() {
        assert_eq!(SCOPE_SINGLETON, "singleton");
        assert_eq!(SCOPE_PROTOTYPE, "prototype");
        assert_eq!(ROLE_APPLICATION, 0);
        assert_eq!(ROLE_SUPPORT, 1);
        assert_eq!(ROLE_INFRASTRUCTURE, 2);
    }

    #[test]
    fn test_bean_class_name_various() {
        let bd1 = TestBeanDefinition::new("MyService");
        assert_eq!(bd1.bean_class_name(), "MyService");

        let bd2 = TestBeanDefinition::new("com.example.MyService");
        assert_eq!(bd2.bean_class_name(), "com.example.MyService");
    }

    #[test]
    fn test_scope_and_lifecycle() {
        let bd = TestBeanDefinition::new("Test")
            .with_scope(Scope::Transient)
            .with_lazy(true)
            .with_primary(true);
        assert_eq!(bd.scope(), Scope::Transient);
        assert!(bd.is_lazy_init());
        assert!(bd.is_primary());
        assert!(bd.is_prototype());
        assert!(!bd.is_singleton());
    }

    // ── Additional coverage for uncovered paths ─────────────────────────────

    #[test]
    fn test_is_fallback_custom() {
        #[derive(Debug)]
        struct FallbackBean;
        impl BeanDefinition for FallbackBean {
            fn bean_name(&self) -> &ComponentKey {
                unimplemented!()
            }
            fn bean_class_name(&self) -> &str {
                "FallbackBean"
            }
            fn scope(&self) -> Scope {
                Scope::Singleton
            }
            fn is_lazy_init(&self) -> bool {
                false
            }
            fn is_primary(&self) -> bool {
                false
            }
            fn is_fallback(&self) -> bool {
                true
            }
        }
        let bd = FallbackBean;
        assert!(bd.is_fallback());
    }

    #[test]
    fn test_is_autowire_candidate_false() {
        #[derive(Debug)]
        struct NoAutowireBean;
        impl BeanDefinition for NoAutowireBean {
            fn bean_name(&self) -> &ComponentKey {
                unimplemented!()
            }
            fn bean_class_name(&self) -> &str {
                "NoAutowireBean"
            }
            fn scope(&self) -> Scope {
                Scope::Singleton
            }
            fn is_lazy_init(&self) -> bool {
                false
            }
            fn is_primary(&self) -> bool {
                false
            }
            fn is_autowire_candidate(&self) -> bool {
                false
            }
        }
        let bd = NoAutowireBean;
        assert!(!bd.is_autowire_candidate());
    }

    #[test]
    fn test_description_custom() {
        #[derive(Debug)]
        struct DescribedBean;
        impl BeanDefinition for DescribedBean {
            fn bean_name(&self) -> &ComponentKey {
                unimplemented!()
            }
            fn bean_class_name(&self) -> &str {
                "DescribedBean"
            }
            fn scope(&self) -> Scope {
                Scope::Singleton
            }
            fn is_lazy_init(&self) -> bool {
                false
            }
            fn is_primary(&self) -> bool {
                false
            }
            fn description(&self) -> Option<&str> {
                Some("A described bean")
            }
        }
        let bd = DescribedBean;
        assert_eq!(bd.description(), Some("A described bean"));
    }

    #[test]
    fn test_parent_name_custom() {
        #[derive(Debug)]
        struct ChildBean;
        impl BeanDefinition for ChildBean {
            fn bean_name(&self) -> &ComponentKey {
                unimplemented!()
            }
            fn bean_class_name(&self) -> &str {
                "ChildBean"
            }
            fn scope(&self) -> Scope {
                Scope::Singleton
            }
            fn is_lazy_init(&self) -> bool {
                false
            }
            fn is_primary(&self) -> bool {
                false
            }
            fn parent_name(&self) -> Option<&str> {
                Some("ParentBean")
            }
        }
        let bd = ChildBean;
        assert_eq!(bd.parent_name(), Some("ParentBean"));
    }

    #[test]
    fn test_factory_bean_name_custom() {
        #[derive(Debug)]
        struct FactoryCreatedBean;
        impl BeanDefinition for FactoryCreatedBean {
            fn bean_name(&self) -> &ComponentKey {
                unimplemented!()
            }
            fn bean_class_name(&self) -> &str {
                "FactoryCreatedBean"
            }
            fn scope(&self) -> Scope {
                Scope::Singleton
            }
            fn is_lazy_init(&self) -> bool {
                false
            }
            fn is_primary(&self) -> bool {
                false
            }
            fn factory_bean_name(&self) -> Option<&str> {
                Some("myFactory")
            }
            fn factory_method_name(&self) -> Option<&str> {
                Some("create")
            }
        }
        let bd = FactoryCreatedBean;
        assert_eq!(bd.factory_bean_name(), Some("myFactory"));
        assert_eq!(bd.factory_method_name(), Some("create"));
    }

    #[test]
    fn test_init_and_destroy_method_names() {
        #[derive(Debug)]
        struct LifecycleBean;
        impl BeanDefinition for LifecycleBean {
            fn bean_name(&self) -> &ComponentKey {
                unimplemented!()
            }
            fn bean_class_name(&self) -> &str {
                "LifecycleBean"
            }
            fn scope(&self) -> Scope {
                Scope::Singleton
            }
            fn is_lazy_init(&self) -> bool {
                false
            }
            fn is_primary(&self) -> bool {
                false
            }
            fn init_method_name(&self) -> Option<&str> {
                Some("init")
            }
            fn destroy_method_name(&self) -> Option<&str> {
                Some("destroy")
            }
        }
        let bd = LifecycleBean;
        assert_eq!(bd.init_method_name(), Some("init"));
        assert_eq!(bd.destroy_method_name(), Some("destroy"));
    }

    #[test]
    fn test_is_abstract_custom() {
        #[derive(Debug)]
        struct AbstractBean;
        impl BeanDefinition for AbstractBean {
            fn bean_name(&self) -> &ComponentKey {
                unimplemented!()
            }
            fn bean_class_name(&self) -> &str {
                "AbstractBean"
            }
            fn scope(&self) -> Scope {
                Scope::Singleton
            }
            fn is_lazy_init(&self) -> bool {
                false
            }
            fn is_primary(&self) -> bool {
                false
            }
            fn is_abstract(&self) -> bool {
                true
            }
        }
        let bd = AbstractBean;
        assert!(bd.is_abstract());
    }

    #[test]
    fn test_resource_description_custom() {
        #[derive(Debug)]
        struct ResourceBean;
        impl BeanDefinition for ResourceBean {
            fn bean_name(&self) -> &ComponentKey {
                unimplemented!()
            }
            fn bean_class_name(&self) -> &str {
                "ResourceBean"
            }
            fn scope(&self) -> Scope {
                Scope::Singleton
            }
            fn is_lazy_init(&self) -> bool {
                false
            }
            fn is_primary(&self) -> bool {
                false
            }
            fn resource_description(&self) -> Option<&str> {
                Some("classpath:config.xml")
            }
        }
        let bd = ResourceBean;
        assert_eq!(bd.resource_description(), Some("classpath:config.xml"));
    }

    #[test]
    fn test_originating_bean_definition_custom() {
        let bd = TestBeanDefinition::new("Test");
        // Default returns None
        assert!(bd.originating_bean_definition().is_none());
    }

    #[test]
    fn test_bean_class_name_internal_returns_self() {
        let bd = TestBeanDefinition::new("com.example.Service");
        assert_eq!(bd.bean_class_name_internal(), Some("com.example.Service"));
    }

    #[test]
    fn test_scope_transient_is_not_singleton() {
        let bd = TestBeanDefinition::new("Test").with_scope(Scope::Transient);
        assert!(!bd.is_singleton());
        assert!(bd.is_prototype());
    }

    #[test]
    fn test_scope_singleton_is_not_prototype() {
        let bd = TestBeanDefinition::new("Test").with_scope(Scope::Singleton);
        assert!(bd.is_singleton());
        assert!(!bd.is_prototype());
    }
}
