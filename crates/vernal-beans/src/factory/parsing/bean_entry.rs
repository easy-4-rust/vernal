//! BeanEntry — 对应 Spring beans.factory.parsing.BeanEntry。
//!
//! Bean 条目，表示一个解析后的 Bean 定义摘要信息。用于在解析阶段
//! 暂存 Bean 的关键元数据（名称、类名、作用域），以便后续注册到容器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.parsing.BeanEntry`。

use std::fmt;

use super::location::Location;

/// Bean 条目。
///
/// 对应 Spring 的 `BeanEntry`。
///
/// 在 Bean 定义解析过程中暂存关键元数据。与完整的 BeanDefinition
/// 相比，BeanEntry 更轻量，主要用于解析阶段的中间表示和事件通知。
///
/// ## Spring 语义
///
/// Spring 中 `BeanEntry` 用于 `ReaderContext` 内部跟踪已解析的
/// Bean 定义。每个 `<bean>` 元素解析后会创建一个 BeanEntry，包含
/// Bean 名称、来源、角色等摘要信息。
///
/// ## 示例
///
/// ```rust
/// use vernal_beans::factory::parsing::bean_entry::BeanEntry;
/// use vernal_beans::factory::parsing::location::Location;
///
/// let entry = BeanEntry::new(
///     "myService",
///     "com.example.MyService",
///     Location::new("beans.xml", 10, 3),
/// );
/// assert_eq!(entry.bean_name(), "myService");
/// assert_eq!(entry.bean_class_name(), "com.example.MyService");
/// ```
#[derive(Debug, Clone)]
pub struct BeanEntry {
    /// Bean 名称。
    bean_name: String,
    /// Bean 类名。
    bean_class_name: String,
    /// 来源位置。
    location: Location,
    /// Bean 角色（0=APPLICATION, 1=SUPPORT, 2=INFRASTRUCTURE）。
    role: i32,
    /// 描述信息。
    description: Option<String>,
}

/// 角色：应用 Bean（用户自定义）。
pub const ROLE_APPLICATION: i32 = 0;

/// 角色：支持 Bean（配置辅助）。
pub const ROLE_SUPPORT: i32 = 1;

/// 角色：基础设施 Bean（框架内部）。
pub const ROLE_INFRASTRUCTURE: i32 = 2;

impl BeanEntry {
    /// 创建一个新的 Bean 条目。
    pub fn new(
        bean_name: impl Into<String>,
        bean_class_name: impl Into<String>,
        location: Location,
    ) -> Self {
        Self {
            bean_name: bean_name.into(),
            bean_class_name: bean_class_name.into(),
            location,
            role: ROLE_APPLICATION,
            description: None,
        }
    }

    /// 设置角色（链式构建）。
    #[must_use]
    pub fn with_role(mut self, role: i32) -> Self {
        self.role = role;
        self
    }

    /// 设置描述信息（链式构建）。
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// 返回 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }

    /// 返回 Bean 类名。
    pub fn bean_class_name(&self) -> &str {
        &self.bean_class_name
    }

    /// 返回来源位置。
    pub fn location(&self) -> &Location {
        &self.location
    }

    /// 返回 Bean 角色。
    pub fn role(&self) -> i32 {
        self.role
    }

    /// 返回描述信息。
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// 是否为应用级 Bean。
    pub fn is_application_bean(&self) -> bool {
        self.role == ROLE_APPLICATION
    }

    /// 是否为基础设施 Bean。
    pub fn is_infrastructure_bean(&self) -> bool {
        self.role == ROLE_INFRASTRUCTURE
    }
}

impl fmt::Display for BeanEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Bean '{}' of type [{}] (from {})",
            self.bean_name, self.bean_class_name, self.location
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation_and_accessors() {
        let loc = Location::new("beans.xml", 10, 3);
        let entry = BeanEntry::new("myService", "com.example.MyService", loc);
        assert_eq!(entry.bean_name(), "myService");
        assert_eq!(entry.bean_class_name(), "com.example.MyService");
        assert_eq!(entry.role(), ROLE_APPLICATION);
        assert!(entry.is_application_bean());
        assert!(!entry.is_infrastructure_bean());
        assert!(entry.description().is_none());
    }

    #[test]
    fn test_with_role() {
        let loc = Location::from_resource("infra.xml");
        let entry = BeanEntry::new("internal", "Internal", loc).with_role(ROLE_INFRASTRUCTURE);
        assert_eq!(entry.role(), ROLE_INFRASTRUCTURE);
        assert!(entry.is_infrastructure_bean());
        assert!(!entry.is_application_bean());
    }

    #[test]
    fn test_display() {
        let loc = Location::new("ctx.xml", 1, 1);
        let entry = BeanEntry::new("bean1", "MyClass", loc);
        let display = format!("{}", entry);
        assert!(display.contains("bean1"));
        assert!(display.contains("MyClass"));
    }
}
