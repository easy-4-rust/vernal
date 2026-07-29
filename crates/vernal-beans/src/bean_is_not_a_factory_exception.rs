//! BeanIsNotAFactoryException — Spring 风格的 Bean 不是 FactoryBean 异常。
//!
//! 对应 Java 类：`org.springframework.beans.factory.BeanIsNotAFactoryException`。
//!
//! 当尝试通过 `&` 前缀获取 FactoryBean 但该 Bean 不是 FactoryBean 时抛出。

use std::error::Error;
use std::fmt;

/// Spring 风格的 Bean 不是 FactoryBean 异常。
///
/// 对应 Spring 的 `BeanIsNotAFactoryException`。
///
/// 当客户端通过 `&beanName` 语法请求 FactoryBean 实例，
/// 但指定名称对应的 Bean 未实现 `FactoryBean` 接口时抛出。
#[derive(Clone, Debug)]
pub struct BeanIsNotAFactoryException {
    /// 被请求的 Bean 名称。
    bean_name: String,
}

impl BeanIsNotAFactoryException {
    /// 创建新的 BeanIsNotAFactoryException。
    pub fn new(bean_name: impl Into<String>) -> Self {
        Self {
            bean_name: bean_name.into(),
        }
    }

    /// 获取被请求的 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }
}

impl fmt::Display for BeanIsNotAFactoryException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Bean '{}' is not a FactoryBean; use '&' prefix only for FactoryBeans",
            self.bean_name
        )
    }
}

impl Error for BeanIsNotAFactoryException {}
