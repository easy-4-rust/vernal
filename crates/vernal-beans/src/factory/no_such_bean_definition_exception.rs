//! NoSuchBeanDefinitionException — 对应 Spring `org.springframework.beans.factory.NoSuchBeanDefinitionException`。
//!
//! 当没有找到指定名称或类型的 Bean 定义时抛出的异常。

use std::fmt;

/// 当没有找到指定名称或类型的 Bean 定义时抛出的异常。
///
/// 对应 Java 类：`org.springframework.beans.factory.NoSuchBeanDefinitionException`。
#[derive(Debug)]
pub struct NoSuchBeanDefinitionException {
    bean_name: Option<String>,
    type_name: Option<String>,
    message: String,
}

impl NoSuchBeanDefinitionException {
    /// 按名称创建异常。
    ///
    /// 对应 Java 构造器：`NoSuchBeanDefinitionException(String name)`
    pub fn by_name(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            message: format!("No bean named '{}' available", name),
            bean_name: Some(name),
            type_name: None,
        }
    }

    /// 按类型创建异常。
    ///
    /// 对应 Java 构造器：`NoSuchBeanDefinitionException(Class<?> type)`
    pub fn by_type(type_name: impl Into<String>) -> Self {
        let type_name = type_name.into();
        Self {
            message: format!("No qualifying bean of type '{}' available", type_name),
            bean_name: None,
            type_name: Some(type_name),
        }
    }

    /// 按名称和类型创建异常。
    ///
    /// 对应 Java 构造器：`NoSuchBeanDefinitionException(String name, Class<?> type)`
    pub fn by_name_and_type(name: impl Into<String>, type_name: impl Into<String>) -> Self {
        let name = name.into();
        let type_name = type_name.into();
        Self {
            message: format!(
                "No qualifying bean of type '{}' available for name '{}'",
                type_name, name
            ),
            bean_name: Some(name),
            type_name: Some(type_name),
        }
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> Option<&str> {
        self.bean_name.as_deref()
    }

    /// 获取类型名称。
    pub fn type_name(&self) -> Option<&str> {
        self.type_name.as_deref()
    }
}

impl fmt::Display for NoSuchBeanDefinitionException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for NoSuchBeanDefinitionException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_by_name() {
        let e = NoSuchBeanDefinitionException::by_name("myBean");
        assert_eq!(e.bean_name(), Some("myBean"));
        assert!(e.type_name().is_none());
        assert!(format!("{}", e).contains("myBean"));
    }

    #[test]
    fn test_by_type() {
        let e = NoSuchBeanDefinitionException::by_type("String");
        assert!(e.bean_name().is_none());
        assert_eq!(e.type_name(), Some("String"));
        assert!(format!("{}", e).contains("String"));
    }

    #[test]
    fn test_by_name_and_type() {
        let e = NoSuchBeanDefinitionException::by_name_and_type("myBean", "String");
        assert_eq!(e.bean_name(), Some("myBean"));
        assert_eq!(e.type_name(), Some("String"));
    }
}
