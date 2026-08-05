//! BeanComponentDefinition — 对应 Spring BeanComponentDefinition。
//!
//! 基于 Bean 定义的组件定义。将一个 BeanDefinition 包装为
//! ComponentDefinition，用于解析阶段的组件注册。
//!
//! 对应 Java 类：`org.springframework.beans.factory.parsing.BeanComponentDefinition`。

use std::fmt;

use super::abstract_component_definition::AbstractComponentDefinition;

/// 基于 Bean 定义的组件定义。
///
/// 对应 Spring 的 `BeanComponentDefinition`。
///
/// 将一个 Bean 定义（Bean 名称、类名等）包装为组件定义，
/// 用于解析阶段的组件注册和事件通知。
///
/// ## Spring 语义
///
/// Spring 的 `BeanComponentDefinition` 是最常用的
/// `ComponentDefinition` 实现，每个 `<bean>` 元素解析后
/// 都会创建一个 `BeanComponentDefinition`。
///
/// ## 示例
///
/// ```rust
/// use vernal_beans::factory::parsing::bean_component_definition::BeanComponentDefinition;
///
/// let def = BeanComponentDefinition::new("myService", "com.example.MyService");
/// assert_eq!(def.bean_name(), "myService");
/// assert_eq!(def.bean_class_name(), "com.example.MyService");
/// ```
#[derive(Debug, Clone)]
pub struct BeanComponentDefinition {
    /// 基类组件定义。
    base: AbstractComponentDefinition,
    /// Bean 类名。
    bean_class_name: String,
    /// 描述信息。
    description: Option<String>,
}

impl BeanComponentDefinition {
    /// 创建一个新的 Bean 组件定义。
    ///
    /// # 参数
    /// - `bean_name` — Bean 名称
    /// - `bean_class_name` — Bean 类名
    pub fn new(bean_name: impl Into<String>, bean_class_name: impl Into<String>) -> Self {
        Self {
            base: AbstractComponentDefinition::new(bean_name),
            bean_class_name: bean_class_name.into(),
            description: None,
        }
    }

    /// 设置描述信息（链式构建）。
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// 返回 Bean 名称。
    pub fn bean_name(&self) -> &str {
        self.base.name()
    }

    /// 返回 Bean 类名。
    pub fn bean_class_name(&self) -> &str {
        &self.bean_class_name
    }

    /// 返回描述信息。
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref().or(self.base.description())
    }

    /// 添加别名。
    pub fn add_alias(&mut self, alias: impl Into<String>) {
        self.base.add_alias(alias);
    }

    /// 返回所有别名。
    pub fn aliases(&self) -> &[String] {
        self.base.aliases()
    }

    /// 返回底层的抽象组件定义。
    pub fn base(&self) -> &AbstractComponentDefinition {
        &self.base
    }
}

impl fmt::Display for BeanComponentDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BeanComponentDefinition: {} ({})",
            self.bean_name(),
            self.bean_class_name()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation_and_accessors() {
        let def = BeanComponentDefinition::new("userService", "com.example.UserService");
        assert_eq!(def.bean_name(), "userService");
        assert_eq!(def.bean_class_name(), "com.example.UserService");
        assert!(def.description().is_none());
    }

    #[test]
    fn test_with_description() {
        let def = BeanComponentDefinition::new("svc", "Svc").with_description("The main service");
        assert_eq!(def.description(), Some("The main service"));
    }

    #[test]
    fn test_aliases() {
        let mut def = BeanComponentDefinition::new("bean1", "MyClass");
        def.add_alias("alias1");
        def.add_alias("alias2");
        assert_eq!(def.aliases().len(), 2);
    }

    #[test]
    fn test_display() {
        let def = BeanComponentDefinition::new("myBean", "MyClass");
        let display = format!("{}", def);
        assert!(display.contains("myBean"));
        assert!(display.contains("MyClass"));
    }
}
