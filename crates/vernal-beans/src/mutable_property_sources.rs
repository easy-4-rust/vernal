//! MutablePropertySources — 可变属性源集合。
//!
//! 对应 Java 类：`org.springframework.core.env.MutablePropertySources`。
//!
//! 在 `PropertySources` 之上提供 `add_first` / `add_last` 等显式优先级操作。

use crate::property_source::PropertySource;
use crate::property_sources::PropertySources;

/// Spring 风格的可变属性源集合。
///
/// 对应 Spring 的 `MutablePropertySources`。
///
/// 包装 `PropertySources` 并额外提供最高/最低优先级的插入操作。
/// `add_first` 使属性源获得最高优先级，`add_last` 使其获得最低优先级。
#[derive(Debug, Clone, Default)]
pub struct MutablePropertySources {
    inner: PropertySources,
}

impl MutablePropertySources {
    /// 创建空的可变属性源集合。
    pub fn new() -> Self {
        Self::default()
    }

    /// 从已有 `PropertySources` 创建。
    pub fn from_property_sources(sources: PropertySources) -> Self {
        Self { inner: sources }
    }

    /// 以最高优先级添加属性源（插入到列表头部）。
    ///
    /// 对应 Spring 的 `addFirst(PropertySource<?>)`。
    pub fn add_first(&mut self, property_source: PropertySource) {
        let name = property_source.name().to_owned();
        self.inner.remove(&name);
        self.inner.insert(0, property_source);
    }

    /// 以最低优先级添加属性源（追加到列表尾部）。
    ///
    /// 对应 Spring 的 `addLast(PropertySource<?>)`。
    pub fn add_last(&mut self, property_source: PropertySource) {
        let name = property_source.name().to_owned();
        self.inner.remove(&name);
        self.inner.add(property_source);
    }

    /// 在指定名称的属性源之前添加。
    ///
    /// 对应 Spring 的 `addBefore(String, PropertySource<?>)`。
    pub fn add_before(
        &mut self,
        relative_property_source_name: &str,
        property_source: PropertySource,
    ) {
        let name = property_source.name().to_owned();
        self.inner.remove(&name);
        self.inner
            .add_before(relative_property_source_name, property_source);
    }

    /// 在指定名称的属性源之后添加。
    ///
    /// 对应 Spring 的 `addAfter(String, PropertySource<?>)`。
    pub fn add_after(
        &mut self,
        relative_property_source_name: &str,
        property_source: PropertySource,
    ) {
        let name = property_source.name().to_owned();
        self.inner.remove(&name);
        self.inner
            .add_after(relative_property_source_name, property_source);
    }

    /// 移除指定名称的属性源。
    pub fn remove(&mut self, name: &str) -> Option<PropertySource> {
        self.inner.remove(name)
    }

    /// 替换同名属性源，返回是否替换成功。
    pub fn replace(&mut self, property_source: PropertySource) -> bool {
        self.inner.replace(property_source)
    }

    /// 获取指定名称的属性源。
    pub fn get(&self, name: &str) -> Option<&PropertySource> {
        self.inner.get(name)
    }

    /// 获取指定名称的属性源的可变引用。
    pub fn get_mut(&mut self, name: &str) -> Option<&mut PropertySource> {
        self.inner.get_mut(name)
    }

    /// 是否包含指定名称的属性源。
    pub fn contains(&self, name: &str) -> bool {
        self.inner.contains(name)
    }

    /// 按优先级返回属性源切片。
    pub fn sources(&self) -> &[PropertySource] {
        self.inner.sources()
    }

    /// 按优先级返回属性源名称列表。
    pub fn names(&self) -> Vec<&str> {
        self.inner.names()
    }

    /// 按优先级查询属性值。
    pub fn get_property(&self, name: &str) -> Option<&str> {
        self.inner.get_property(name)
    }

    /// 属性源数量。
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// 清空所有属性源。
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// 获取内部 `PropertySources` 的引用。
    pub fn as_property_sources(&self) -> &PropertySources {
        &self.inner
    }

    /// 消费并返回内部 `PropertySources`。
    pub fn into_property_sources(self) -> PropertySources {
        self.inner
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
    fn test_add_first_last() {
        let mut mutable = MutablePropertySources::new();
        mutable.add_last(ps("low", &[("k", "1")]));
        mutable.add_first(ps("high", &[("k", "2")]));
        assert_eq!(mutable.names(), vec!["high", "low"]);
        assert_eq!(mutable.get_property("k"), Some("2"));
    }

    #[test]
    fn test_promote_with_add_first() {
        let mut mutable = MutablePropertySources::new();
        mutable.add_first(ps("a", &[("k", "1")]));
        mutable.add_first(ps("b", &[("k", "2")]));
        // 再次把 a 提升到最前，应去重而非重复
        mutable.add_first(ps("a", &[("k", "3")]));
        assert_eq!(mutable.len(), 2);
        assert_eq!(mutable.names(), vec!["a", "b"]);
        assert_eq!(mutable.get_property("k"), Some("3"));
    }
}
