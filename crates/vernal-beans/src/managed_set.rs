//! ManagedSet — Spring 风格的受管集合。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.ManagedSet`。
//!
//! 包装 `HashSet<T>` 并附带来源描述，用于在 Bean 定义中表达无序集合属性值。

use std::collections::HashSet;
use std::hash::Hash;

/// 受管集合。
///
/// 对应 Spring 的 `ManagedSet<E>`。
///
/// 泛型 `T` 必须 `Eq + Hash`。语义与 `ManagedList` 对称，底层使用 `HashSet`。
#[derive(Debug, Clone)]
pub struct ManagedSet<T>
where
    T: Eq + Hash,
{
    /// 内部存储。
    inner: HashSet<T>,
    /// 来源描述。
    source_name: Option<String>,
    /// 元素类型名。
    element_type_name: Option<String>,
}

impl<T> ManagedSet<T>
where
    T: Eq + Hash,
{
    /// 创建空的受管集合。
    pub fn new() -> Self {
        Self {
            inner: HashSet::new(),
            source_name: None,
            element_type_name: None,
        }
    }

    /// 从已有 `HashSet` 构造。
    pub fn from_set(set: HashSet<T>) -> Self {
        Self {
            inner: set,
            source_name: None,
            element_type_name: None,
        }
    }

    /// 消耗自身，返回内部 `HashSet`。
    pub fn into_set(self) -> HashSet<T> {
        self.inner
    }

    /// 元素数量。
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// 插入元素，返回是否新增（`HashSet::insert` 语义）。
    pub fn insert(&mut self, value: T) -> bool {
        self.inner.insert(value)
    }

    /// 移除元素，返回是否之前存在。
    pub fn remove(&mut self, value: &T) -> bool {
        self.inner.remove(value)
    }

    /// 是否包含元素。
    pub fn contains(&self, value: &T) -> bool {
        self.inner.contains(value)
    }

    /// 获取内部引用。
    pub fn as_set(&self) -> &HashSet<T> {
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

    /// 设置元素类型名。
    pub fn with_element_type_name(mut self, name: impl Into<String>) -> Self {
        self.element_type_name = Some(name.into());
        self
    }

    /// 获取元素类型名。
    pub fn element_type_name(&self) -> Option<&str> {
        self.element_type_name.as_deref()
    }
}

impl<T> Default for ManagedSet<T>
where
    T: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<HashSet<T>> for ManagedSet<T>
where
    T: Eq + Hash,
{
    fn from(set: HashSet<T>) -> Self {
        Self::from_set(set)
    }
}

impl<T> From<ManagedSet<T>> for HashSet<T>
where
    T: Eq + Hash,
{
    fn from(managed: ManagedSet<T>) -> Self {
        managed.into_set()
    }
}
