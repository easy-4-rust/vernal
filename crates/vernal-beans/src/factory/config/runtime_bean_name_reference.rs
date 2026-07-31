//! RuntimeBeanNameReference — 对应 Spring `org.springframework.beans.factory.config.RuntimeBeanNameReference`。
//!
//! 运行时 Bean 名称引用，表示需要在运行时解析的 Bean 名称引用。

use std::fmt;

/// 运行时 Bean 名称引用。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.RuntimeBeanNameReference`。
///
/// 与 `RuntimeBeanReference` 类似，但表示的是 Bean 名称而非 Bean 实例的引用。
/// 在运行时通过名称查找 Bean。
///
/// ## 使用场景
///
/// - 需要延迟解析 Bean 名称的场景
/// - 配置中引用其他 Bean 名称的场景
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeBeanNameReference {
    /// 引用的 Bean 名称。
    source_bean_name: String,
    /// 目标 Bean 名称（可选）。
    target_bean_name: Option<String>,
}

impl RuntimeBeanNameReference {
    /// 创建新的运行时 Bean 名称引用。
    pub fn new(source_bean_name: impl Into<String>) -> Self {
        Self {
            source_bean_name: source_bean_name.into(),
            target_bean_name: None,
        }
    }

    /// 创建带目标名称的引用。
    pub fn with_target(
        source_bean_name: impl Into<String>,
        target_bean_name: impl Into<String>,
    ) -> Self {
        Self {
            source_bean_name: source_bean_name.into(),
            target_bean_name: Some(target_bean_name.into()),
        }
    }

    /// 获取源 Bean 名称。
    pub fn source_bean_name(&self) -> &str {
        &self.source_bean_name
    }

    /// 获取目标 Bean 名称。
    pub fn target_bean_name(&self) -> Option<&str> {
        self.target_bean_name.as_deref()
    }
}

impl fmt::Display for RuntimeBeanNameReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.target_bean_name {
            Some(target) => write!(f, "{} -> {}", self.source_bean_name, target),
            None => write!(f, "{}", self.source_bean_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_bean_name_reference_new() {
        let reference = RuntimeBeanNameReference::new("myBean");
        assert_eq!(reference.source_bean_name(), "myBean");
        assert!(reference.target_bean_name().is_none());
        assert_eq!(format!("{}", reference), "myBean");
    }

    #[test]
    fn test_runtime_bean_name_reference_with_target() {
        let reference = RuntimeBeanNameReference::with_target("sourceBean", "targetBean");
        assert_eq!(reference.source_bean_name(), "sourceBean");
        assert_eq!(reference.target_bean_name(), Some("targetBean"));
        assert_eq!(format!("{}", reference), "sourceBean -> targetBean");
    }

    #[test]
    fn test_runtime_bean_name_reference_clone_and_eq() {
        let ref1 = RuntimeBeanNameReference::new("bean1");
        let ref2 = ref1.clone();
        assert_eq!(ref1, ref2);

        let ref3 = RuntimeBeanNameReference::with_target("bean1", "bean2");
        assert_ne!(ref1, ref3);
    }
}
