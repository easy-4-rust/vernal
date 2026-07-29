//! BeanWiringInfo — Spring 风格的 Bean 装配信息。
//!
//! 对应 Java 类：`org.springframework.beans.factory.wiring.BeanWiringInfo`。
//!
//! 描述一个 Bean 的装配方式（按名/按类型自动装配），
//! 由 `BeanConfigurerSupport` / `@Configurable` 机制使用。

/// 默认按名自动装配的标志。
pub const AUTOWIRE_BY_NAME: &str = "byName";
/// 默认按类型自动装配的标志。
pub const AUTOWIRE_BY_TYPE: &str = "byType";

/// Bean 装配信息。
///
/// 对应 Spring 的 `BeanWiringInfo`。
///
/// 记录如何装配一个对象：
/// - 是否自动装配，以及按名还是按类型
/// - 显式指定的 Bean 名称（非自动装配场景）
/// - 是否注入默认依赖（`@Configurable(autowire = Autowire.BY_TYPE)` 时）
#[derive(Debug, Clone)]
pub struct BeanWiringInfo {
    /// 装配模式：`"byName"` / `"byType"`，或其他自定义标识。
    wiring_mode: String,
    /// 显式指定的 Bean 名称（非自动装配时使用）。
    bean_name: Option<String>,
    /// 是否注入默认依赖。
    default_dependency: bool,
    /// 是否忽略依赖解析失败。
    ignore_unresolved: bool,
}

impl BeanWiringInfo {
    /// 创建按名自动装配的信息。
    ///
    /// 对应 Spring 的 `BeanWiringInfo(int mode)`（`AUTOWIRE_BY_NAME`）。
    pub fn autowire_by_name() -> Self {
        Self {
            wiring_mode: AUTOWIRE_BY_NAME.to_string(),
            bean_name: None,
            default_dependency: false,
            ignore_unresolved: false,
        }
    }

    /// 创建按类型自动装配的信息。
    ///
    /// 对应 Spring 的 `BeanWiringInfo(int mode)`（`AUTOWIRE_BY_TYPE`）。
    pub fn autowire_by_type(default_dependency: bool) -> Self {
        Self {
            wiring_mode: AUTOWIRE_BY_TYPE.to_string(),
            bean_name: None,
            default_dependency,
            ignore_unresolved: false,
        }
    }

    /// 创建指向特定 Bean 名称的非自动装配信息。
    ///
    /// 对应 Spring 的 `BeanWiringInfo(String beanName, boolean defaultDependency)`。
    pub fn new(bean_name: impl Into<String>, default_dependency: bool) -> Self {
        Self {
            wiring_mode: String::new(),
            bean_name: Some(bean_name.into()),
            default_dependency,
            ignore_unresolved: false,
        }
    }

    /// 获取装配模式。
    pub fn wiring_mode(&self) -> &str {
        &self.wiring_mode
    }

    /// 获取显式 Bean 名称。
    pub fn bean_name(&self) -> Option<&str> {
        self.bean_name.as_deref()
    }

    /// 是否注入默认依赖。
    pub fn indicates_default_dependency(&self) -> bool {
        self.default_dependency
    }

    /// 是否忽略未解析依赖。
    pub fn ignore_unresolved(&self) -> bool {
        self.ignore_unresolved
    }

    /// 设置是否忽略未解析依赖。
    pub fn set_ignore_unresolved(&mut self, ignore: bool) {
        self.ignore_unresolved = ignore;
    }

    /// 是否为自动装配（按名或按类型）。
    ///
    /// 对应 Spring 的 `indicatesAutowiring()`。
    pub fn indicates_autowiring(&self) -> bool {
        matches!(
            self.wiring_mode.as_str(),
            AUTOWIRE_BY_NAME | AUTOWIRE_BY_TYPE
        )
    }

    /// 获取依赖名称。
    ///
    /// 对应 Spring 的 `getDependencyName()`：
    /// - 自动装配且非默认依赖时，返回模式名（`"byName"` / `"byType"`）
    /// - 否则返回 `None`
    pub fn get_dependency_name(&self) -> Option<&str> {
        if self.indicates_autowiring() && !self.default_dependency {
            Some(&self.wiring_mode)
        } else {
            None
        }
    }
}
