//! BeanCurrentlyInCreationException — 对应 Spring `org.springframework.beans.factory.BeanCurrentlyInCreationException`。
//!
//! Bean 正在创建中（循环依赖）时抛出的异常。

use std::fmt;

/// Bean 正在创建中（循环依赖）时抛出的异常。
///
/// 对应 Java 类：`org.springframework.beans.factory.BeanCurrentlyInCreationException`。
///
/// 当检测到循环依赖时抛出此异常。
#[derive(Debug)]
pub struct BeanCurrentlyInCreationException {
    bean_name: String,
}

impl BeanCurrentlyInCreationException {
    /// 创建一个新的 BeanCurrentlyInCreationException。
    ///
    /// 对应 Java 构造器：`BeanCurrentlyInCreationException(String beanName)`
    pub fn new(bean_name: impl Into<String>) -> Self {
        Self {
            bean_name: bean_name.into(),
        }
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }
}

impl fmt::Display for BeanCurrentlyInCreationException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Requested bean is currently in creation: Is there an unresolvable circular reference for '{}'?",
            self.bean_name
        )
    }
}

impl std::error::Error for BeanCurrentlyInCreationException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let e = BeanCurrentlyInCreationException::new("myBean");
        assert_eq!(e.bean_name(), "myBean");
    }

    #[test]
    fn test_display() {
        let e = BeanCurrentlyInCreationException::new("myBean");
        assert!(format!("{}", e).contains("myBean"));
        assert!(format!("{}", e).contains("circular reference"));
    }
}
