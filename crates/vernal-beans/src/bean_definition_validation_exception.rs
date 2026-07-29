//! BeanDefinitionValidationException — Spring 风格的 Bean 定义验证异常。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionValidationException`。
//!
//! 当 Bean 定义验证失败时抛出，表示 Bean 定义的结构或内容无效。

use std::error::Error;
use std::fmt;

/// Spring 风格的 Bean 定义验证异常。
///
/// 对应 Spring 的 `BeanDefinitionValidationException`。
///
/// 当 Bean 定义不符合预期结构或约束条件时抛出。
#[derive(Clone, Debug)]
pub struct BeanDefinitionValidationException {
    /// 验证错误消息。
    message: String,
}

impl BeanDefinitionValidationException {
    /// 创建新的 BeanDefinitionValidationException。
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for BeanDefinitionValidationException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bean definition validation failed: {}", self.message)
    }
}

impl Error for BeanDefinitionValidationException {}
