//! PropertySources — Spring 风格的属性源集合。
//!
//! 对应 Java 类：`org.springframework.core.env.PropertySources`。
//!
//! 维护一组有序的 `PropertySource`，提供按优先级的查询与增删替换能力。

use crate::property_source::PropertySource;

/// Spring 风格的属性源集合。
///
/// 对应 Spring 的 `PropertySources`。
///
/// 维护一组按优先级排序的 `PropertySource`：列表前部的属性源优先级更高，
/// 在同名属性查询时优先返回。`MutablePropertySources` 提供额外的
/// `add_first` / `add_last` 操作。
#[derive(Debug, Clone, Default)]
pub struct PropertySources {
    sources: Vec<PropertySource>,
}

impl PropertySources {
    /// 创建空的属性源集合。
    pub fn new() -> Self {
        Self::default()
    }

    /// 在末尾追加一个属性源（最低优先级）。
    pub fn add(&mut self, property_source: PropertySource) {
        self.sources.push(property_source);
    }

    /// 在指定名称的属性源之前插入一个属性源。
    ///
    /// 若 `before` 不存在，则追加到末尾。
    pub fn add_before(&mut self, before: &str, property_source: PropertySource) {
        if let Some(pos) = self.position_of(before) {
            self.sources.insert(pos, property_source);
        } else {
            self.sources.push(property_source);
        }
    }

    /// 在指定名称的属性源之后插入一个属性源。
    ///
    /// 若 `after` 不存在，则追加到末尾。
    pub fn add_after(&mut self, after: &str, property_source: PropertySource) {
        if let Some(pos) = self.position_of(after) {
            self.sources.insert(pos + 1, property_source);
        } else {
            self.sources.push(property_source);
        }
    }

    /// 替换同名的属性源。
    ///
    /// 返回是否发生了替换。
    pub fn replace(&mut self, property_source: PropertySource) -> bool {
        let name = property_source.name().to_owned();
        if let Some(pos) = self.position_of(&name) {
            self.sources[pos] = property_source;
            true
        } else {
            false
        }
    }

    /// 在指定位置插入属性源（保留顺序）。
    pub fn insert(&mut self, index: usize, property_source: PropertySource) {
        if index >= self.sources.len() {
            self.sources.push(property_source);
        } else {
            self.sources.insert(index, property_source);
        }
    }

    /// 移除指定名称的属性源。
    ///
    /// 返回被移除的属性源。
    pub fn remove(&mut self, name: &str) -> Option<PropertySource> {
        if let Some(pos) = self.position_of(name) {
            Some(self.sources.remove(pos))
        } else {
            None
        }
    }

    /// 获取指定名称的属性源。
    pub fn get(&self, name: &str) -> Option<&PropertySource> {
        self.sources.iter().find(|s| s.name() == name)
    }

    /// 获取指定名称的属性源的可变引用。
    pub fn get_mut(&mut self, name: &str) -> Option<&mut PropertySource> {
        self.sources.iter_mut().find(|s| s.name() == name)
    }

    /// 是否包含指定名称的属性源。
    pub fn contains(&self, name: &str) -> bool {
        self.sources.iter().any(|s| s.name() == name)
    }

    /// 返回属性源名称的有序列表（高优先级在前）。
    pub fn names(&self) -> Vec<&str> {
        self.sources.iter().map(|s| s.name()).collect()
    }

    /// 按优先级返回属性源切片。
    pub fn sources(&self) -> &[PropertySource] {
        &self.sources
    }

    /// 按优先级返回首个包含指定属性的属性源中的属性值。
    ///
    /// 对应 Spring 的 `PropertySourcesPropertyResolver` 查询逻辑：
    /// 自高优先级向低优先级遍历，返回首个命中值。
    pub fn get_property(&self, name: &str) -> Option<&str> {
        self.sources.iter().find_map(|s| s.get_property(name))
    }

    /// 按优先级返回是否包含指定属性。
    pub fn contains_property(&self, name: &str) -> bool {
        self.sources.iter().any(|s| s.contains_property(name))
    }

    /// 属性源数量。
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    /// 清空所有属性源。
    pub fn clear(&mut self) {
        self.sources.clear();
    }

    fn position_of(&self, name: &str) -> Option<usize> {
        self.sources.iter().position(|s| s.name() == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ps(name: &str, pairs: &[(&str, &str)]) -> PropertySource {
        PropertySource::from_pairs(
            name,
            pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())),
        )
    }

    #[test]
    fn test_priority_lookup() {
        let mut sources = PropertySources::new();
        sources.add(ps("high", &[("k", "1")]));
        sources.add(ps("low", &[("k", "2")]));
        // 第一个加入的优先
        assert_eq!(sources.get_property("k"), Some("1"));
    }

    #[test]
    fn test_remove_replace() {
        let mut sources = PropertySources::new();
        sources.add(ps("a", &[("x", "1")]));
        assert!(sources.contains("a"));
        assert!(sources.replace(ps("a", &[("x", "2")])));
        assert_eq!(sources.get("a").unwrap().get_property("x"), Some("2"));

        let removed = sources.remove("a");
        assert!(removed.is_some());
        assert!(!sources.contains("a"));
    }
}
