//! MapFactoryBean — 对应 Spring `org.springframework.beans.factory.config.MapFactoryBean`。
//!
//! Map 工厂 Bean，用于创建 Map 集合。

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

/// Map 工厂 Bean。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.MapFactoryBean`。
///
/// 用于创建 `Map` 集合的 FactoryBean。
/// 在 Spring XML 配置中使用 `<map>` 标签时，会创建此 Bean。
///
/// ## 使用场景
///
/// - XML 配置中的 `<map>` 标签
/// - 注入 Map 类型的 Bean 属性
/// - 创建键值对集合
#[derive(Debug)]
pub struct MapFactoryBean {
    /// Map 中的键值对。
    source_map: Vec<(Arc<dyn Any + Send + Sync>, Arc<dyn Any + Send + Sync>)>,
    /// 目标 Map 的键类型名。
    target_key_type: Option<String>,
    /// 目标 Map 的值类型名。
    target_value_type: Option<String>,
}

impl MapFactoryBean {
    /// 创建新的 MapFactoryBean。
    pub fn new() -> Self {
        Self {
            source_map: Vec::new(),
            target_key_type: None,
            target_value_type: None,
        }
    }

    /// 设置源 Map。
    pub fn set_source_map(
        &mut self,
        source_map: Vec<(Arc<dyn Any + Send + Sync>, Arc<dyn Any + Send + Sync>)>,
    ) {
        self.source_map = source_map;
    }

    /// 获取源 Map。
    pub fn source_map(&self) -> &[(Arc<dyn Any + Send + Sync>, Arc<dyn Any + Send + Sync>)] {
        &self.source_map
    }

    /// 添加键值对。
    pub fn add_entry(
        &mut self,
        key: Arc<dyn Any + Send + Sync>,
        value: Arc<dyn Any + Send + Sync>,
    ) {
        self.source_map.push((key, value));
    }

    /// 设置目标键类型。
    pub fn set_target_key_type(&mut self, target_type: impl Into<String>) {
        self.target_key_type = Some(target_type.into());
    }

    /// 设置目标值类型。
    pub fn set_target_value_type(&mut self, target_type: impl Into<String>) {
        self.target_value_type = Some(target_type.into());
    }

    /// 获取目标键类型。
    pub fn target_key_type(&self) -> Option<&str> {
        self.target_key_type.as_deref()
    }

    /// 获取目标值类型。
    pub fn target_value_type(&self) -> Option<&str> {
        self.target_value_type.as_deref()
    }

    /// 创建 Map 实例（返回 Vec，因为 Arc<dyn Any> 不实现 Eq/Hash）。
    pub fn create_map(&self) -> Vec<(Arc<dyn Any + Send + Sync>, Arc<dyn Any + Send + Sync>)> {
        self.source_map.clone()
    }

    /// 获取 Map 中的条目数量。
    pub fn size(&self) -> usize {
        self.source_map.len()
    }
}

impl Default for MapFactoryBean {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_factory_bean_new() {
        let factory = MapFactoryBean::new();
        assert!(factory.source_map().is_empty());
        assert!(factory.target_key_type().is_none());
        assert!(factory.target_value_type().is_none());
        assert_eq!(factory.size(), 0);
    }

    #[test]
    fn test_map_factory_bean_with_entries() {
        let mut factory = MapFactoryBean::new();
        factory.add_entry(
            Arc::new(String::from("key1")),
            Arc::new(100i32),
        );
        factory.add_entry(
            Arc::new(String::from("key2")),
            Arc::new(200i32),
        );

        assert_eq!(factory.size(), 2);
        let map = factory.create_map();
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_map_factory_bean_target_types() {
        let mut factory = MapFactoryBean::new();
        factory.set_target_key_type("java.lang.String");
        factory.set_target_value_type("java.lang.Integer");
        assert_eq!(factory.target_key_type(), Some("java.lang.String"));
        assert_eq!(factory.target_value_type(), Some("java.lang.Integer"));
    }
}
