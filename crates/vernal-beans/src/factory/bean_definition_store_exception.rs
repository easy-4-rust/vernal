//! BeanDefinitionStoreException — 对应 Spring `org.springframework.beans.factory.BeanDefinitionStoreException`。
//!
//! Bean 定义存储异常。

use std::fmt;

/// Bean 定义存储异常。
///
/// 对应 Java 类：`org.springframework.beans.factory.BeanDefinitionStoreException`。
#[derive(Debug)]
pub struct BeanDefinitionStoreException {
    message: String,
    resource_description: Option<String>,
    bean_name: Option<String>,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl BeanDefinitionStoreException {
    /// 创建一个新的 BeanDefinitionStoreException。
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            resource_description: None,
            bean_name: None,
            source: None,
        }
    }

    /// 创建一个带有资源描述的 BeanDefinitionStoreException。
    pub fn with_resource(
        resource_description: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            message: message.into(),
            resource_description: Some(resource_description.into()),
            bean_name: None,
            source: None,
        }
    }

    /// 创建一个带有资源描述和原因的 BeanDefinitionStoreException。
    pub fn with_cause(
        resource_description: impl Into<String>,
        message: impl Into<String>,
        cause: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            resource_description: Some(resource_description.into()),
            bean_name: None,
            source: Some(Box::new(cause)),
        }
    }

    /// 获取资源描述。
    pub fn resource_description(&self) -> Option<&str> {
        self.resource_description.as_deref()
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> Option<&str> {
        self.bean_name.as_deref()
    }

    /// 设置 Bean 名称。
    pub fn set_bean_name(&mut self, bean_name: impl Into<String>) {
        self.bean_name = Some(bean_name.into());
    }
}

impl fmt::Display for BeanDefinitionStoreException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref resource) = self.resource_description {
            write!(
                f,
                "Failed to read bean definition from '{}': {}",
                resource, self.message
            )
        } else {
            write!(f, "Bean definition store error: {}", self.message)
        }
    }
}

impl std::error::Error for BeanDefinitionStoreException {
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
        let e = BeanDefinitionStoreException::new("parse error");
        assert_eq!(format!("{}", e), "Bean definition store error: parse error");
    }

    #[test]
    fn test_with_resource() {
        let e = BeanDefinitionStoreException::with_resource("classpath:beans.xml", "invalid XML");
        assert_eq!(e.resource_description(), Some("classpath:beans.xml"));
        assert!(format!("{}", e).contains("classpath:beans.xml"));
    }

    #[test]
    fn test_with_cause() {
        let cause = std::io::Error::new(std::io::ErrorKind::Other, "IO error");
        let e = BeanDefinitionStoreException::with_cause("beans.xml", "read failed", cause);
        assert!(e.source().is_some());
    }

    #[test]
    fn test_set_bean_name() {
        let mut e = BeanDefinitionStoreException::new("error");
        assert!(e.bean_name().is_none());
        e.set_bean_name("myBean");
        assert_eq!(e.bean_name(), Some("myBean"));
    }
}
