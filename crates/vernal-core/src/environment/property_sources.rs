//! 属性源集合。
//!
//! 对标 Spring `org.springframework.core.env.PropertySources`。
//! 按插入顺序管理多个属性源，按优先级查找。

use super::property_source::PropertySource;

/// 属性源集合。
///
/// 对应 Java: org.springframework.core.env.PropertySources
pub struct PropertySources {
    sources: Vec<Box<dyn PropertySource>>,
}

impl PropertySources {
    /// 创建空的属性源集合。
    #[must_use]
    pub fn new() -> Self {
        Self { sources: Vec::new() }
    }

    /// 在最高优先级位置加入属性源。
    pub fn add_first(&mut self, source: Box<dyn PropertySource>) {
        self.sources.insert(0, source);
    }

    /// 在最低优先级位置加入属性源。
    pub fn add_last(&mut self, source: Box<dyn PropertySource>) {
        self.sources.push(source);
    }

    /// 按优先级顺序查找属性值。
    pub fn get_property(&self, key: &str) -> Option<String> {
        for source in &self.sources {
            if let Some(value) = source.get_property(key) {
                return Some(value);
            }
        }
        None
    }

    /// 检查属性是否存在。
    pub fn contains_property(&self, key: &str) -> bool {
        self.get_property(key).is_some()
    }

    /// 返回属性源数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }
}

impl Default for PropertySources {
    fn default() -> Self {
        Self::new()
    }
}
