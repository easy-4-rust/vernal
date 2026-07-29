//! BeanIsAbstractException — Spring 风格的 Bean 是抽象异常。
//!
//! 对应 Java 类：`org.springframework.beans.factory.BeanIsAbstractException`。
//!
//! 当尝试通过 `getBean` 获取一个标记为 `abstract` 的 Bean 时抛出。

use std::error::Error;
use std::fmt;

/// Spring 风格的 Bean 是抽象异常。
///
/// 对应 Spring 的 `BeanIsAbstractException`。
///
/// 表示客户端尝试实例化一个被标记为 abstract 的 Bean 定义。
/// 抽象 Bean 定义仅作为子类定义的模板使用，不能直接获取。
#[derive(Clone, Debug)]
pub struct BeanIsAbstractException {
    /// 被请求的 Bean 名称。
    bean_name: String,
}

impl BeanIsAbstractException {
    /// 创建新的 BeanIsAbstractException。
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

impl fmt::Display for BeanIsAbstractException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Bean '{}' is abstract; it cannot be directly instantiated",
            self.bean_name
        )
    }
}

impl Error for BeanIsAbstractException {}
