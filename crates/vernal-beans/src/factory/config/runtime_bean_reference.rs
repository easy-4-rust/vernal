//! RuntimeBeanReference — 对应 Spring `org.springframework.beans.factory.config.RuntimeBeanReference`。
//!
//! 运行时 Bean 引用，表示在运行时需要解析的 Bean 引用。

use std::fmt;

/// 运行时 Bean 引用。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.RuntimeBeanReference`。
///
/// 在 Bean 定义解析阶段使用，表示需要在运行时从容器中获取的 Bean 引用。
/// 与 `BeanReference` 不同，`RuntimeBeanReference` 用于 BeanDefinition 属性值中。
///
/// ## 使用场景
///
/// - XML 配置中的 `<ref bean="..."/>` 解析结果
/// - 注解 `@Autowired` 中的 Bean 引用
/// - Java 配置 `@Bean` 方法中的参数引用
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeBeanReference {
    /// 引用的 Bean 名称。
    bean_name: String,
    /// 是否来自父容器。
    from_parent: bool,
}

impl RuntimeBeanReference {
    /// 创建新的运行时 Bean 引用。
    pub fn new(bean_name: impl Into<String>) -> Self {
        Self {
            bean_name: bean_name.into(),
            from_parent: false,
        }
    }

    /// 创建来自父容器的 Bean 引用。
    pub fn from_parent(bean_name: impl Into<String>) -> Self {
        Self {
            bean_name: bean_name.into(),
            from_parent: true,
        }
    }

    /// 获取引用的 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }

    /// 是否来自父容器。
    pub fn is_from_parent(&self) -> bool {
        self.from_parent
    }
}

impl fmt::Display for RuntimeBeanReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.from_parent {
            write!(f, "parent '{}'", self.bean_name)
        } else {
            write!(f, "ref '{}'", self.bean_name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_bean_reference_new() {
        let reference = RuntimeBeanReference::new("myBean");
        assert_eq!(reference.bean_name(), "myBean");
        assert!(!reference.is_from_parent());
        assert_eq!(format!("{}", reference), "ref 'myBean'");
    }

    #[test]
    fn test_runtime_bean_reference_from_parent() {
        let reference = RuntimeBeanReference::from_parent("parentBean");
        assert_eq!(reference.bean_name(), "parentBean");
        assert!(reference.is_from_parent());
        assert_eq!(format!("{}", reference), "parent 'parentBean'");
    }

    #[test]
    fn test_runtime_bean_reference_clone_and_eq() {
        let ref1 = RuntimeBeanReference::new("bean1");
        let ref2 = ref1.clone();
        assert_eq!(ref1, ref2);

        let ref3 = RuntimeBeanReference::new("bean2");
        assert_ne!(ref1, ref3);
    }
}
