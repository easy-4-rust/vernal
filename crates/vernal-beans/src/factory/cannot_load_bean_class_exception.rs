//! CannotLoadBeanClassException — 对应 Spring `org.springframework.beans.factory.CannotLoadBeanClassException`。
//!
//! 无法加载 Bean 类时抛出的异常。

use std::fmt;

/// 无法加载 Bean 类时抛出的异常。
///
/// 对应 Java 类：`org.springframework.beans.factory.CannotLoadBeanClassException`。
#[derive(Debug)]
pub struct CannotLoadBeanClassException {
    resource_description: Option<String>,
    bean_name: String,
    bean_class_name: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl CannotLoadBeanClassException {
    /// 创建一个新的 CannotLoadBeanClassException。
    pub fn new(
        resource_description: Option<impl Into<String>>,
        bean_name: impl Into<String>,
        bean_class_name: impl Into<String>,
        cause: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            resource_description: resource_description.map(|s| s.into()),
            bean_name: bean_name.into(),
            bean_class_name: bean_class_name.into(),
            source: Some(Box::new(cause)),
        }
    }

    /// 获取资源描述。
    pub fn resource_description(&self) -> Option<&str> {
        self.resource_description.as_deref()
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }

    /// 获取 Bean 类名称。
    pub fn bean_class_name(&self) -> &str {
        &self.bean_class_name
    }
}

impl fmt::Display for CannotLoadBeanClassException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cannot find class '{}' for bean with name '{}' defined in {}",
            self.bean_class_name,
            self.bean_name,
            self.resource_description.as_deref().unwrap_or("unknown")
        )
    }
}

impl std::error::Error for CannotLoadBeanClassException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_new() {
        let cause = std::io::Error::new(std::io::ErrorKind::NotFound, "class not found");
        let e = CannotLoadBeanClassException::new(
            Some("classpath:beans.xml"),
            "myBean",
            "com.example.MyClass",
            cause,
        );
        assert_eq!(e.bean_name(), "myBean");
        assert_eq!(e.bean_class_name(), "com.example.MyClass");
        assert!(e.source().is_some());
    }

    #[test]
    fn test_display() {
        let cause = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let e = CannotLoadBeanClassException::new(
            Some("beans.xml"),
            "myBean",
            "com.example.MyClass",
            cause,
        );
        assert!(format!("{}", e).contains("com.example.MyClass"));
    }
}
