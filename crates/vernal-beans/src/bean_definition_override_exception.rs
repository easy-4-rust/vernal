//! BeanDefinitionOverrideException — Spring 风格的 Bean 定义覆盖异常。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionOverrideException`。
//!
//! 当尝试注册一个与现有 Bean 定义同名的定义且不允许覆盖时抛出。

use std::error::Error;
use std::fmt;

/// Spring 风格的 Bean 定义覆盖异常。
///
/// 对应 Spring 的 `BeanDefinitionOverrideException`。
///
/// 当 `BeanDefinitionOverridingStrategy` 决定禁止覆盖时抛出此异常。
#[derive(Clone, Debug)]
pub struct BeanDefinitionOverrideException {
    /// 尝试注册的 Bean 名称。
    bean_name: String,
    /// 详细信息。
    message: String,
}

impl BeanDefinitionOverrideException {
    /// 创建新的 BeanDefinitionOverrideException。
    pub fn new(bean_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            bean_name: bean_name.into(),
            message: message.into(),
        }
    }

    /// 获取冲突的 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }
}

impl fmt::Display for BeanDefinitionOverrideException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Bean definition override not allowed for bean '{}': {}",
            self.bean_name, self.message
        )
    }
}

impl Error for BeanDefinitionOverrideException {}
