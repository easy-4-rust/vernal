//! ManagedProperties — Spring 风格的受管属性集合。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.ManagedProperties`。
//!
//! 包装 `HashMap<String, String>` 用于在 Bean 定义中表达 `Properties` 类型属性值。

use std::collections::HashMap;

/// 受管属性集合。
///
/// 对应 Spring 的 `ManagedProperties`（继承自 `Properties`）。
///
/// 用于表达 `<props>` 标签或 `Properties` 类型注入值。提供来源追溯以及
/// 与原生 `HashMap<String, String>` 的互转。
#[derive(Debug, Clone)]
pub struct ManagedProperties {
    /// 内部存储。
    inner: HashMap<String, String>,
    /// 来源描述。
    source_name: Option<String>,
    /// 是否合并（对应 Spring 的 `merge` 属性）。
    merge_enabled: bool,
}

impl ManagedProperties {
    /// 创建空的属性集合。
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            source_name: None,
            merge_enabled: false,
        }
    }

    /// 从已有 `HashMap` 构造。
    pub fn from_map(map: HashMap<String, String>) -> Self {
        Self {
            inner: map,
            source_name: None,
            merge_enabled: false,
        }
    }

    /// 消耗自身，返回内部 `HashMap`。
    pub fn into_map(self) -> HashMap<String, String> {
        self.inner
    }

    /// 条目数量。
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// 设置属性。
    pub fn set_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.inner.insert(key.into(), value.into());
    }

    /// 获取属性。
    pub fn get_property(&self, key: &str) -> Option<&str> {
        self.inner.get(key).map(|s| s.as_str())
    }

    /// 移除属性。
    pub fn remove_property(&mut self, key: &str) -> Option<String> {
        self.inner.remove(key)
    }

    /// 是否包含属性。
    pub fn contains_property(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    /// 获取所有键。
    pub fn property_names(&self) -> Vec<&str> {
        self.inner.keys().map(|s| s.as_str()).collect()
    }

    /// 获取内部引用。
    pub fn as_map(&self) -> &HashMap<String, String> {
        &self.inner
    }

    /// 设置来源描述。
    pub fn with_source_name(mut self, name: impl Into<String>) -> Self {
        self.source_name = Some(name.into());
        self
    }

    /// 获取来源描述。
    pub fn source_name(&self) -> Option<&str> {
        self.source_name.as_deref()
    }

    /// 启用/禁用合并。
    pub fn set_merge_enabled(&mut self, enabled: bool) {
        self.merge_enabled = enabled;
    }

    /// 是否启用合并。
    pub fn is_merge_enabled(&self) -> bool {
        self.merge_enabled
    }
}

impl Default for ManagedProperties {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HashMap<String, String>> for ManagedProperties {
    fn from(map: HashMap<String, String>) -> Self {
        Self::from_map(map)
    }
}

impl From<ManagedProperties> for HashMap<String, String> {
    fn from(managed: ManagedProperties) -> Self {
        managed.into_map()
    }
}
