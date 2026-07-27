//! BeanDefinition — Spring 风格的 Bean 元数据 trait。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.BeanDefinition`。
//!
//! 描述一个 Bean 实例的元数据，包括类型信息、作用域、依赖、初始化方法等。
//! 这是 vernal-beans 与 Spring IoC 容器语义对齐的核心接口。

use std::any::Any;
use std::fmt;

use crate::component_scope::Scope;
use crate::component_key::ComponentKey;

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
