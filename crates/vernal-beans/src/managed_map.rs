//! ManagedMap — Spring 风格的受管映射。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.ManagedMap`。
//!
//! 包装 `HashMap<K, V>` 并附带来源描述，用于在 Bean 定义中表达键值对属性值。

use std::collections::HashMap;
use std::hash::Hash;

/// 受管映射。
///
/// 对应 Spring 的 `ManagedMap<K, V>`。
///
/// 键类型 `K` 必须 `Eq + Hash`。提供来源追溯与键/值类型名记录。
#[derive(Debug, Clone)]
pub struct ManagedMap<K, V>
where
    K: Eq + Hash,
{
    /// 内部存储。
    inner: HashMap<K, V>,
    /// 来源描述。
    source_name: Option<String>,
    /// 键类型名。
    key_type_name: Option<String>,
    /// 值类型名。
    value_type_name: Option<String>,
}

impl<K, V> ManagedMap<K, V>
where
    K: Eq + Hash,
{
    /// 创建空的受管映射。
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            source_name: None,
            key_type_name: None,
            value_type_name: None,
        }
    }

    /// 从已有 `HashMap` 构造。
    pub fn from_map(map: HashMap<K, V>) -> Self {
        Self {
            inner: map,
            source_name: None,
            key_type_name: None,
            value_type_name: None,
        }
    }

    /// 消耗自身，返回内部 `HashMap`。
    pub fn into_map(self) -> HashMap<K, V> {
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

    /// 插入键值对，返回旧值（`HashMap::insert` 语义）。
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    /// 移除键，返回旧值。
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.inner.remove(key)
    }

    /// 按键查找。
    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }

    /// 是否包含键。
    pub fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }

    /// 获取内部引用。
    pub fn as_map(&self) -> &HashMap<K, V> {
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

    /// 设置键类型名。
    pub fn with_key_type_name(mut self, name: impl Into<String>) -> Self {
        self.key_type_name = Some(name.into());
        self
    }

    /// 获取键类型名。
    pub fn key_type_name(&self) -> Option<&str> {
        self.key_type_name.as_deref()
    }

    /// 设置值类型名。
    pub fn with_value_type_name(mut self, name: impl Into<String>) -> Self {
        self.value_type_name = Some(name.into());
        self
    }

    /// 获取值类型名。
    pub fn value_type_name(&self) -> Option<&str> {
        self.value_type_name.as_deref()
    }
}

impl<K, V> Default for ManagedMap<K, V>
where
    K: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> From<HashMap<K, V>> for ManagedMap<K, V>
where
    K: Eq + Hash,
{
    fn from(map: HashMap<K, V>) -> Self {
        Self::from_map(map)
    }
}

impl<K, V> From<ManagedMap<K, V>> for HashMap<K, V>
where
    K: Eq + Hash,
{
    fn from(managed: ManagedMap<K, V>) -> Self {
        managed.into_map()
    }
}
