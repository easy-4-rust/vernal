//! NamedBean — 对应 Spring `org.springframework.beans.factory.NamedBean`。
//!
//! 命名 Bean 接口。

/// 命名 Bean 接口。
///
/// 对应 Java 接口：`org.springframework.beans.factory.NamedBean`。
///
/// 提供了获取 Bean 名称的能力。
pub trait NamedBean: Send + Sync {
    /// 获取 Bean 的名称。
    ///
    /// 对应 Java 方法：`String getBeanName()`
    fn bean_name(&self) -> &str;
}

/// NamedBean 的简单实现。
#[derive(Debug, Clone)]
pub struct SimpleNamedBean {
    name: String,
}

impl SimpleNamedBean {
    /// 创建一个新的实例。
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl NamedBean for SimpleNamedBean {
    fn bean_name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_named_bean() {
        let bean = SimpleNamedBean::new("myBean");
        assert_eq!(bean.bean_name(), "myBean");
    }
}
