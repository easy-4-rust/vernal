//! 属性源抽象 trait。
//!
//! 对标 Spring `org.springframework.core.env.PropertySource`。

/// 属性源抽象 trait。
///
/// 对应 Java: org.springframework.core.env.PropertySource
pub trait PropertySource: Send + Sync {
    /// 返回属性源名称。
    ///
    /// 对应 Java: `PropertySource#getName()`
    fn name(&self) -> &str;

    /// 获取属性值。
    ///
    /// 对应 Java: `PropertySource#getProperty(String)`
    fn get_property(&self, key: &str) -> Option<String>;

    /// 检查属性是否存在（默认实现）。
    ///
    /// 对应 Java: `PropertySource#containsProperty(String)`
    fn contains_property(&self, key: &str) -> bool {
        self.get_property(key).is_some()
    }
}
