//! 可枚举属性源契约。
//!
//! 对标 Spring `org.springframework.core.env.EnumerablePropertySource`。

use super::PropertySource;

/// 可枚举属性源契约。
///
/// 对应 Java: org.springframework.core.env.EnumerablePropertySource
///
/// Spring 语义：可列出全部属性名的 `PropertySource`（对标
/// `SystemEnvironmentPropertySource` / `MapPropertySource` 的能力）。
pub trait EnumerablePropertySource: PropertySource {
    /// 返回全部属性名。
    ///
    /// 对应 Java: `EnumerablePropertySource#getPropertyNames()`
    fn property_names(&self) -> Vec<String>;
}

/// 任意可枚举属性源都可按名称枚举（辅助函数）。
#[must_use]
pub fn property_names_of(source: &dyn EnumerablePropertySource) -> Vec<String> {
    source.property_names()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::MapPropertySource;

    #[test]
    fn map_source_lists_names() {
        // A 类（合同对齐）：对标 Spring 属性名枚举
        let mut map = std::collections::HashMap::new();
        map.insert("a".to_string(), "1".to_string());
        map.insert("b".to_string(), "2".to_string());
        let source = MapPropertySource::new("test", map);
        let mut names = source.property_names();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }
}
