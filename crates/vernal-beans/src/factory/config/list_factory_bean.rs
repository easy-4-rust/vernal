//! ListFactoryBean — 对应 Spring `org.springframework.beans.factory.config.ListFactoryBean`。
//!
//! List 工厂 Bean，用于创建 List 集合。

use std::any::Any;
use std::sync::Arc;

/// List 工厂 Bean。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.ListFactoryBean`。
///
/// 用于创建 `List` 集合的 FactoryBean。
/// 在 Spring XML 配置中使用 `<list>` 标签时，会创建此 Bean。
///
/// ## 使用场景
///
/// - XML 配置中的 `<list>` 标签
/// - 注入 List 类型的 Bean 属性
/// - 创建类型安全的集合
#[derive(Debug)]
pub struct ListFactoryBean {
    /// List 中的元素。
    source_list: Vec<Arc<dyn Any + Send + Sync>>,
    /// 目标 List 的元素类型名。
    target_list_type: Option<String>,
}

impl ListFactoryBean {
    /// 创建新的 ListFactoryBean。
    pub fn new() -> Self {
        Self {
            source_list: Vec::new(),
            target_list_type: None,
        }
    }

    /// 设置源 List。
    pub fn set_source_list(&mut self, source_list: Vec<Arc<dyn Any + Send + Sync>>) {
        self.source_list = source_list;
    }

    /// 获取源 List。
    pub fn source_list(&self) -> &[Arc<dyn Any + Send + Sync>] {
        &self.source_list
    }

    /// 设置目标 List 类型。
    pub fn set_target_list_type(&mut self, target_type: impl Into<String>) {
        self.target_list_type = Some(target_type.into());
    }

    /// 获取目标 List 类型。
    pub fn target_list_type(&self) -> Option<&str> {
        self.target_list_type.as_deref()
    }

    /// 创建 List 实例。
    pub fn create_list(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
        self.source_list.clone()
    }

    /// 获取 List 中的元素数量。
    pub fn size(&self) -> usize {
        self.source_list.len()
    }
}

impl Default for ListFactoryBean {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_factory_bean_new() {
        let factory = ListFactoryBean::new();
        assert!(factory.source_list().is_empty());
        assert!(factory.target_list_type().is_none());
        assert_eq!(factory.size(), 0);
    }

    #[test]
    fn test_list_factory_bean_with_elements() {
        let mut factory = ListFactoryBean::new();
        let elements: Vec<Arc<dyn Any + Send + Sync>> = vec![
            Arc::new(String::from("hello")),
            Arc::new(42i32),
            Arc::new(true),
        ];
        factory.set_source_list(elements);

        assert_eq!(factory.size(), 3);
        let list = factory.create_list();
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_list_factory_bean_target_type() {
        let mut factory = ListFactoryBean::new();
        factory.set_target_list_type("java.util.ArrayList");
        assert_eq!(factory.target_list_type(), Some("java.util.ArrayList"));
    }
}
