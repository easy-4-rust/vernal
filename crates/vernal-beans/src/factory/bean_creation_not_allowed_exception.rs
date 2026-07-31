//! BeanCreationNotAllowedException — 对应 Spring `org.springframework.beans.factory.BeanCreationNotAllowedException`。
//!
//! Bean 创建被禁止时抛出的异常（通常在单例销毁阶段）。

use std::fmt;

/// Bean 创建被禁止时抛出的异常。
///
/// 对应 Java 类：`org.springframework.beans.factory.BeanCreationNotAllowedException`。
///
/// 当在单例销毁阶段尝试创建新 Bean 时抛出此异常。
#[derive(Debug)]
pub struct BeanCreationNotAllowedException {
    bean_name: String,
    message: String,
}

impl BeanCreationNotAllowedException {
    /// 创建一个新的 BeanCreationNotAllowedException。
    pub fn new(bean_name: impl Into<String>) -> Self {
        let bean_name = bean_name.into();
        Self {
            message: format!(
                "Singleton bean creation not allowed while the singletons of this factory are in destruction \
                (Do not request a bean from a BeanFactory in a destroy method implementation!): '{}'",
                bean_name
            ),
            bean_name,
        }
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }
}

impl fmt::Display for BeanCreationNotAllowedException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BeanCreationNotAllowedException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let e = BeanCreationNotAllowedException::new("myBean");
        assert_eq!(e.bean_name(), "myBean");
        assert!(format!("{}", e).contains("myBean"));
        assert!(format!("{}", e).contains("Singleton bean creation not allowed"));
    }
}
