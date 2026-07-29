//! BeanDefinitionStoreException — Spring 风格的 Bean 定义存储异常。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionStoreException`。
//!
//! 当从外部资源加载 Bean 定义失败时抛出。包含资源描述和原始原因。

use std::error::Error;
use std::fmt;

/// Spring 风格的 Bean 定义存储异常。
///
/// 对应 Spring 的 `BeanDefinitionStoreException`。
///
/// 表示从 Bean 定义资源（XML、注解、Groovy 等）加载定义时出现问题。
#[derive(Debug)]
pub struct BeanDefinitionStoreException {
    /// 资源描述（如文件名、类路径位置）。
    resource_description: String,
    /// 详细消息。
    message: String,
    /// 原始原因（可选）。
    cause: Option<Box<dyn Error + Send + Sync>>,
}

impl BeanDefinitionStoreException {
    /// 创建新的 BeanDefinitionStoreException（无原因）。
    pub fn new(resource_description: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            resource_description: resource_description.into(),
            message: message.into(),
            cause: None,
        }
    }

    /// 创建带原因的 BeanDefinitionStoreException。
    pub fn with_cause(
        resource_description: impl Into<String>,
        message: impl Into<String>,
        cause: Box<dyn Error + Send + Sync>,
    ) -> Self {
        Self {
            resource_description: resource_description.into(),
            message: message.into(),
            cause: Some(cause),
        }
    }

    /// 获取资源描述。
    pub fn resource_description(&self) -> &str {
        &self.resource_description
    }
}

impl fmt::Display for BeanDefinitionStoreException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error loading bean definitions from '{}': {}",
            self.resource_description, self.message
        )?;
        if let Some(ref cause) = self.cause {
            write!(f, "; caused by: {cause}")?;
        }
        Ok(())
    }
}

impl Error for BeanDefinitionStoreException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.as_ref().and_then(|c| c.source())
    }
}
