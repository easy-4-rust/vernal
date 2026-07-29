//! FactoryBeanNotInitializedException — Spring 风格的 FactoryBean 未初始化异常。
//!
//! 对应 Java 类：`org.springframework.beans.factory.FactoryBeanNotInitializedException`。
//!
//! 当 FactoryBean 尚未完全初始化时尝试获取其管理的对象时抛出。

use std::error::Error;
use std::fmt;

/// Spring 风格的 FactoryBean 未初始化异常。
///
/// 对应 Spring 的 `FactoryBeanNotInitializedException`。
///
/// 当 FactoryBean 尚未完全初始化时，调用 `get_object()` 或类似方法抛出此异常。
/// 典型场景是在 FactoryBean 自身的初始化方法中尝试获取该 FactoryBean 管理的对象。
#[derive(Clone, Debug)]
pub struct FactoryBeanNotInitializedException {
    /// 详细信息。
    message: String,
}

impl FactoryBeanNotInitializedException {
    /// 创建新的 FactoryBeanNotInitializedException。
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Default for FactoryBeanNotInitializedException {
    fn default() -> Self {
        Self {
            message: "FactoryBean is not fully initialized yet".to_string(),
        }
    }
}

impl fmt::Display for FactoryBeanNotInitializedException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FactoryBean not initialized: {}", self.message)
    }
}

impl Error for FactoryBeanNotInitializedException {}
