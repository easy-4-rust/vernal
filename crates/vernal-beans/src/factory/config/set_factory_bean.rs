//! SetFactoryBean — 对应 Spring `org.springframework.beans.factory.config.SetFactoryBean`。
//!
//! Set 工厂 Bean，用于创建 Set 集合。

use std::any::Any;
use std::collections::HashSet;
use std::sync::Arc;

/// Set 工厂 Bean。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.SetFactoryBean`。
///
/// 用于创建 `Set` 集合的 FactoryBean。
/// 在 Spring XML 配置中使用 `<set>` 标签时，会创建此 Bean。
///
/// ## 使用场景
///
/// - XML 配置中的 `<set>` 标签
/// - 注入 Set 类型的 Bean 属性
/// - 创建无重复元素的集合
#[derive(Debug)]
pub struct SetFactoryBean {
    /// Set 中的元素。
    source_set: Vec<Arc<dyn Any + Send + Sync>>,
    /// 目标 Set 的元素类型名。
    target_set_type: Option<String>,
}

impl SetFactoryBean {
    /// 创建新的 SetFactoryBean。
    pub fn new() -> Self {
        Self {
            source_set: Vec::new(),
            target_set_type: None,
        }
    }

    /// 设置源 Set。
    pub fn set_source_set(&mut self, source_set: Vec<Arc<dyn Any + Send + Sync>>) {
        self.source_set = source_set;
    }

    /// 获取源 Set。
    pub fn source_set(&self) -> &[Arc<dyn Any + Send + Sync>] {
        &self.source_set
    }

    /// 设置目标 Set 类型。
    pub fn set_target_set_type(&mut self, target_type: impl Into<String>) {
        self.target_set_type = Some(target_type.into());
    }

    /// 获取目标 Set 类型。
    pub fn target_set_type(&self) -> Option<&str> {
        self.target_set_type.as_deref()
    }

    /// 创建 Set 实例（返回 Vec，因为 Arc<dyn Any> 不实现 Eq/Hash）。
    pub fn create_set(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
        self.source_set.clone()
    }

    /// 获取 Set 中的元素数量。
    pub fn size(&self) -> usize {
        self.source_set.len()
    }
}

impl Default for SetFactoryBean {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_factory_bean_new() {
        let factory = SetFactoryBean::new();
        assert!(factory.source_set().is_empty());
        assert!(factory.target_set_type().is_none());
        assert_eq!(factory.size(), 0);
    }

    #[test]
    fn test_set_factory_bean_with_elements() {
        let mut factory = SetFactoryBean::new();
        let elements: Vec<Arc<dyn Any + Send + Sync>> = vec![
            Arc::new(String::from("hello")),
            Arc::new(42i32),
            Arc::new(true),
        ];
        factory.set_source_set(elements);

        assert_eq!(factory.size(), 3);
        let set = factory.create_set();
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn test_set_factory_bean_target_type() {
        let mut factory = SetFactoryBean::new();
        factory.set_target_set_type("java.util.HashSet");
        assert_eq!(factory.target_set_type(), Some("java.util.HashSet"));
    }
}
