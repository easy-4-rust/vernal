//! BeanReference — 对应 Spring `org.springframework.beans.factory.config.BeanReference`。
//!
//! Bean 引用，表示对另一个 Bean 的引用。

use std::fmt;

/// Bean 引用。
///
/// 对应 Java 接口：`org.springframework.beans.factory.config.BeanReference`。
///
/// 表示对另一个 Bean 的引用。
#[derive(Debug, Clone)]
pub struct BeanReference {
    bean_name: String,
}

impl BeanReference {
    /// 创建一个新的实例。
    pub fn new(bean_name: impl Into<String>) -> Self {
        Self { bean_name: bean_name.into() }
    }

    /// 获取Bean名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }
}

impl fmt::Display for BeanReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ref '{}'", self.bean_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bean_reference() {
        let r = BeanReference::new("myBean");
        assert_eq!(r.bean_name(), "myBean");
        assert_eq!(format!("{}", r), "ref 'myBean'");
    }
}
