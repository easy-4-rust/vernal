//! BeanCreationException — 对应 Spring `org.springframework.beans.factory.BeanCreationException`。
//!
//! Bean 创建失败时抛出的异常。

use std::fmt;

/// Bean 创建失败时抛出的异常。
///
/// 对应 Java 类：`org.springframework.beans.factory.BeanCreationException`。
///
/// 当 Bean 工厂在创建 Bean 实例时遇到错误时抛出此异常。
#[derive(Debug)]
pub struct BeanCreationException {
    message: String,
    resource_description: Option<String>,
    bean_name: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl BeanCreationException {
    /// 创建一个新的 BeanCreationException。
    ///
    /// 对应 Java 构造器：`BeanCreationException(String msg)`
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            resource_description: None,
            bean_name: String::new(),
            source: None,
        }
    }

    /// 创建一个新的 BeanCreationException，指定 Bean 名称。
    ///
    /// 对应 Java 构造器：`BeanCreationException(String beanName, String msg)`
    pub fn with_bean_name(bean_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            resource_description: None,
            bean_name: bean_name.into(),
            source: None,
        }
    }

    /// 创建一个新的 BeanCreationException，指定 Bean 名称和根原因。
    ///
    /// 对应 Java 构造器：`BeanCreationException(String beanName, String msg, Throwable cause)`
    pub fn with_cause(
        bean_name: impl Into<String>,
        message: impl Into<String>,
        cause: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            resource_description: None,
            bean_name: bean_name.into(),
            source: Some(Box::new(cause)),
        }
    }

    /// 获取资源描述。
    pub fn resource_description(&self) -> Option<&str> {
        self.resource_description.as_deref()
    }

    /// 设置资源描述。
    pub fn set_resource_description(&mut self, description: impl Into<String>) {
        self.resource_description = Some(description.into());
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }

    /// 获取异常消息。
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for BeanCreationException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.bean_name.is_empty() {
            write!(f, "Error creating bean: {}", self.message)
        } else {
            write!(
                f,
                "Error creating bean with name '{}': {}",
                self.bean_name, self.message
            )
        }
    }
}

impl std::error::Error for BeanCreationException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_new() {
        let e = BeanCreationException::new("test error");
        assert_eq!(e.message(), "test error");
        assert_eq!(e.bean_name(), "");
    }

    #[test]
    fn test_with_bean_name() {
        let e = BeanCreationException::with_bean_name("myBean", "creation failed");
        assert_eq!(e.bean_name(), "myBean");
        assert_eq!(e.message(), "creation failed");
    }

    #[test]
    fn test_with_cause() {
        let cause = std::io::Error::new(std::io::ErrorKind::Other, "inner error");
        let e = BeanCreationException::with_cause("myBean", "outer error", cause);
        assert!(e.source().is_some());
    }

    #[test]
    fn test_display() {
        let e = BeanCreationException::with_bean_name("myBean", "failed");
        assert_eq!(
            format!("{}", e),
            "Error creating bean with name 'myBean': failed"
        );
    }

    #[test]
    fn test_display_no_bean_name() {
        let e = BeanCreationException::new("general error");
        assert_eq!(format!("{}", e), "Error creating bean: general error");
    }

    #[test]
    fn test_resource_description() {
        let mut e = BeanCreationException::new("test");
        assert!(e.resource_description().is_none());
        e.set_resource_description("classpath:my-config.xml");
        assert_eq!(e.resource_description(), Some("classpath:my-config.xml"));
    }
}
