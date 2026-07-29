//! BeanDefinitionResource — Spring 风格的 Bean 定义资源 trait。
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionResource`。
use std::fmt;

/// Spring 风格的 Bean 定义资源 trait。
pub trait BeanDefinitionResource: fmt::Debug + Send + Sync {
    fn description(&self) -> &str;
    fn location(&self) -> &str;
}
