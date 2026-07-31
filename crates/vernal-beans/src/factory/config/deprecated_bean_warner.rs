//! DeprecatedBeanWarner — 对应 Spring `org.springframework.beans.factory.config.DeprecatedBeanWarner`。
//!
//! 废弃 Bean 警告器。

/// 废弃 Bean 警告器。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.DeprecatedBeanWarner`。
///
/// 用于在访问废弃 Bean 时发出警告。
#[derive(Debug)]
pub struct DeprecatedBeanWarner {
    deprecated_beans: Vec<String>,
}

impl DeprecatedBeanWarner {
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self { deprecated_beans: Vec::new() }
    }

    /// 添加deprecatedBean。
    pub fn add_deprecated_bean(&mut self, bean_name: impl Into<String>) {
        self.deprecated_beans.push(bean_name.into());
    }

    /// 判断是否deprecated。
    pub fn is_deprecated(&self, bean_name: &str) -> bool {
        self.deprecated_beans.iter().any(|n| n == bean_name)
    }

    /// 执行deprecated_beans操作。
    pub fn deprecated_beans(&self) -> &[String] {
        &self.deprecated_beans
    }
}

impl Default for DeprecatedBeanWarner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warner() {
        let mut warner = DeprecatedBeanWarner::new();
        assert!(!warner.is_deprecated("myBean"));

        warner.add_deprecated_bean("myBean");
        assert!(warner.is_deprecated("myBean"));
        assert_eq!(warner.deprecated_beans().len(), 1);
    }
}
