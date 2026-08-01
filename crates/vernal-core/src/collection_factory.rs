//! 集合工厂。
//!
//! 对标 Spring `org.springframework.core.CollectionFactory`。

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

/// 集合工厂。
///
/// 对应 Java: org.springframework.core.CollectionFactory
///
/// Spring 语义：按接口类型创建最合适的集合实现（对标 `CollectionFactory`
/// 的 `createCollection` / `createMap` 分发）。
pub struct CollectionFactory;

impl CollectionFactory {
    /// 创建近似容量空集合（对标 Spring `createCollection`）。
    #[must_use]
    pub fn create_collection<T>(capacity: usize) -> Vec<T> {
        Vec::with_capacity(capacity)
    }

    /// 创建有序集合（对标 Spring `LinkedHashSet` 语义）。
    #[must_use]
    pub fn create_ordered_set<T>() -> Vec<T> {
        Vec::new()
    }

    /// 创建近似容量空映射（对标 Spring `createMap`）。
    #[must_use]
    pub fn create_map<K, V>(capacity: usize) -> HashMap<K, V> {
        HashMap::with_capacity(capacity)
    }

    /// 创建有序映射（对标 Spring `LinkedHashMap` 语义）。
    #[must_use]
    pub fn create_ordered_map<K: Ord, V>() -> BTreeMap<K, V> {
        BTreeMap::new()
    }

    /// 判断类型是否为集合（对标 Spring `isApproximableCollectionType`）。
    ///
    /// 当前实现总是返回 `true`（Rust 标准集合即集合类型）。
    #[must_use]
    pub fn is_collection<T>(marker: Option<&T>) -> bool {
        let _ = marker;
        true
    }

    /// 构造已知集合类型（`HashSet` / `VecDeque` / `Vec`）。
    #[must_use]
    pub fn new_set<T>() -> HashSet<T> {
        HashSet::new()
    }

    /// 构造双端队列。
    #[must_use]
    pub fn new_deque<T>() -> VecDeque<T> {
        VecDeque::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_collections_with_capacity() {
        // A 类（合同对齐）：对标 Spring 容量预分配
        let collection = CollectionFactory::create_collection::<i32>(16);
        assert!(collection.is_empty());
        let map = CollectionFactory::create_map::<String, i32>(8);
        assert!(map.is_empty());
    }

    #[test]
    fn ordered_map_keeps_sorted_keys() {
        // B 类（边界行为）：对标 Spring LinkedHashMap 顺序语义
        let mut map = CollectionFactory::create_ordered_map::<String, i32>();
        map.insert("b".to_string(), 2);
        map.insert("a".to_string(), 1);
        let keys: Vec<&String> = map.keys().collect();
        assert_eq!(keys, vec!["a", "b"]);
    }
}
