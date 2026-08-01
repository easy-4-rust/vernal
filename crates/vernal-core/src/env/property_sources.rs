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
    #[must_use]
    pub fn get_property(&self, key: &str) -> Option<String> {
        for source in &self.sources {
            if let Some(value) = source.get_property(key) {
                return Some(value);
            }
        }
        None
    }

    /// 检查属性是否存在。
    #[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::property_source::PropertySource;

    /// A simple in-memory property source for testing.
    struct MockPropertySource {
        name: String,
        props: std::collections::HashMap<String, String>,
    }

    impl MockPropertySource {
        fn new(name: &str, props: Vec<(&str, &str)>) -> Self {
            Self {
                name: name.to_string(),
                props: props.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            }
        }
    }

    impl PropertySource for MockPropertySource {
        fn name(&self) -> &str {
            &self.name
        }
        fn get_property(&self, key: &str) -> Option<String> {
            self.props.get(key).cloned()
        }
    }

    #[test]
    fn new_creates_empty_collection() {
        let sources = PropertySources::new();
        assert!(sources.is_empty());
        assert_eq!(sources.len(), 0);
    }

    #[test]
    fn add_last_appends_to_end() {
        let mut sources = PropertySources::new();
        sources.add_last(Box::new(MockPropertySource::new("a", vec![("k", "a-val")])));
        sources.add_last(Box::new(MockPropertySource::new("b", vec![("k", "b-val")])));
        assert_eq!(sources.len(), 2);
        // First source wins (added last = lowest priority, but it's at index 1)
        // get_property iterates from index 0, so "a" (index 0) wins
        assert_eq!(sources.get_property("k"), Some("a-val".to_string()));
    }

    #[test]
    fn add_first_inserts_at_beginning() {
        let mut sources = PropertySources::new();
        sources.add_last(Box::new(MockPropertySource::new("a", vec![("k", "a-val")])));
        sources.add_first(Box::new(MockPropertySource::new("b", vec![("k", "b-val")])));
        // "b" is now at index 0 (highest priority)
        assert_eq!(sources.get_property("k"), Some("b-val".to_string()));
    }

    #[test]
    fn get_property_returns_first_match() {
        let mut sources = PropertySources::new();
        sources.add_last(Box::new(MockPropertySource::new("src1", vec![("a", "1"), ("b", "2")])));
        sources.add_last(Box::new(MockPropertySource::new("src2", vec![("b", "3"), ("c", "4")])));
        assert_eq!(sources.get_property("a"), Some("1".to_string()));
        assert_eq!(sources.get_property("b"), Some("2".to_string())); // src1 wins
        assert_eq!(sources.get_property("c"), Some("4".to_string()));
        assert!(sources.get_property("d").is_none());
    }

    #[test]
    fn contains_property_returns_true_when_present() {
        let mut sources = PropertySources::new();
        sources.add_last(Box::new(MockPropertySource::new("src", vec![("key", "val")])));
        assert!(sources.contains_property("key"));
        assert!(!sources.contains_property("missing"));
    }

    #[test]
    fn default_creates_empty() {
        let sources = PropertySources::default();
        assert!(sources.is_empty());
    }

    #[test]
    fn property_source_name_returns_source_name() {
        // 对标 Spring: PropertySource.getName() 返回属性源名称
        // 覆盖行 87-89: MockPropertySource::name() 方法
        let source = MockPropertySource::new("test-source", vec![("k", "v")]);
        assert_eq!(source.name(), "test-source");
    }

    #[test]
    fn property_source_contains_property_delegate_to_get() {
        // 对标 Spring: PropertySource.containsProperty() 默认实现
        let source = MockPropertySource::new("src", vec![("exists", "val")]);
        assert!(source.contains_property("exists"));
        assert!(!source.contains_property("missing"));
    }
}
