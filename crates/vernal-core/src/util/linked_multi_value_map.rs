//! 保持插入顺序的多值 Map。
//!
//! 对标 Spring `org.springframework.util.LinkedMultiValueMap`，基于 LinkedHashMap
//! 保证键的迭代顺序与插入顺序一致（对标 Spring 用 `LinkedHashMap<K, List<V>>` 包装）。

use super::multi_value_map::MultiValueMapTrait;

/// 保持插入顺序的多值 Map。
///
/// 对标 Spring `LinkedMultiValueMap<K, V>`。
///
/// 内部用 `Vec<(K, Vec<V>)>` 存储以保持零依赖，同时保证迭代顺序。
/// 当键已存在时，值追加到已有列表；键不存在时追加到末尾。
///
/// # 示例
///
/// ```
/// use vernal_core::util::LinkedMultiValueMap;
/// use vernal_core::util::MultiValueMapTrait;
///
/// let mut map = LinkedMultiValueMap::new();
/// map.add("fruit", "apple");
/// map.add("fruit", "banana");
/// map.add("color", "red");
///
/// // 迭代顺序与插入顺序一致
/// let keys: Vec<&str> = map.iter_keys().copied().collect();
/// assert_eq!(keys, vec!["fruit", "color"]);
/// assert_eq!(map.get_first(&"fruit"), Some(&"apple"));
/// ```
pub struct LinkedMultiValueMap<K: PartialEq, V> {
    entries: Vec<(K, Vec<V>)>,
}

impl<K: PartialEq, V> LinkedMultiValueMap<K, V> {
    /// 创建空的有序多值 Map。
    #[must_use]
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// 键数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 按插入顺序迭代键。
    pub fn iter_keys(&self) -> impl Iterator<Item = &K> {
        self.entries.iter().map(|(k, _)| k)
    }

    /// 按插入顺序迭代（键, 值列表）。
    pub fn iter(&self) -> impl Iterator<Item = (&K, &Vec<V>)> {
        self.entries.iter().map(|(k, vs)| (k, vs))
    }

    /// 查找键的位置。
    fn index_of(&self, key: &K) -> Option<usize> {
        self.entries.iter().position(|(k, _)| k == key)
    }

    /// 移除键及其所有值。
    pub fn remove(&mut self, key: &K) -> Option<Vec<V>> {
        self.index_of(key).map(|i| self.entries.remove(i).1)
    }

    /// 清空。
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// 是否包含键。
    #[must_use]
    pub fn contains_key(&self, key: &K) -> bool {
        self.index_of(key).is_some()
    }

    /// 转换为单值 Map（取每个键的第一个值）。
    ///
    /// 对标 Spring `toSingleValueMap()`。
    #[must_use]
    pub fn to_single_value_map(&self) -> Vec<(K, V)>
    where
        K: Clone,
        V: Clone,
    {
        self.entries
            .iter()
            .filter_map(|(k, vs)| vs.first().map(|v| (k.clone(), v.clone())))
            .collect()
    }

    /// 深拷贝（克隆所有值列表）。
    ///
    /// 对标 Spring `LinkedMultiValueMap.deepCopy()`（since 4.2）。
    #[must_use]
    pub fn deep_copy(&self) -> Self
    where
        K: Clone,
        V: Clone,
    {
        Self {
            entries: self
                .entries
                .iter()
                .map(|(k, vs)| (k.clone(), vs.clone()))
                .collect(),
        }
    }
}

impl<K: PartialEq, V> Default for LinkedMultiValueMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: PartialEq, V> Clone for LinkedMultiValueMap<K, V>
where
    K: Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        self.deep_copy()
    }
}

impl<K: PartialEq + std::fmt::Debug, V: std::fmt::Debug> std::fmt::Debug
    for LinkedMultiValueMap<K, V>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.entries.iter().map(|(k, vs)| (k, vs)))
            .finish()
    }
}

impl<K: PartialEq, V> MultiValueMapTrait<K, V> for LinkedMultiValueMap<K, V> {
    fn get_first(&self, key: &K) -> Option<&V> {
        self.index_of(key)
            .and_then(|i| self.entries[i].1.first())
    }

    fn add(&mut self, key: K, value: V) {
        if let Some(i) = self.index_of(&key) {
            self.entries[i].1.push(value);
        } else {
            self.entries.push((key, vec![value]));
        }
    }

    fn set(&mut self, key: K, value: V) {
        if let Some(i) = self.index_of(&key) {
            self.entries[i].1 = vec![value];
        } else {
            self.entries.push((key, vec![value]));
        }
    }

    fn get_all(&self, key: &K) -> Option<&[V]> {
        self.index_of(key).map(|i| self.entries[i].1.as_slice())
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn contains_key(&self, key: &K) -> bool {
        self.index_of(key).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_insertion_order() {
        let mut map = LinkedMultiValueMap::new();
        map.add("c", 3);
        map.add("a", 1);
        map.add("b", 2);
        let keys: Vec<&str> = map.iter_keys().copied().collect();
        assert_eq!(keys, vec!["c", "a", "b"]);
    }

    #[test]
    fn add_appends_to_existing() {
        let mut map = LinkedMultiValueMap::new();
        map.add("k", "v1");
        map.add("k", "v2");
        assert_eq!(map.get_all(&"k"), Some(&["v1", "v2"][..]));
        assert_eq!(map.len(), 1); // 只有一个键
    }

    #[test]
    fn set_replaces_all() {
        let mut map = LinkedMultiValueMap::new();
        map.add("k", "v1");
        map.add("k", "v2");
        map.set("k", "new");
        assert_eq!(map.get_all(&"k"), Some(&["new"][..]));
    }

    #[test]
    fn get_first() {
        let mut map = LinkedMultiValueMap::new();
        map.add("k", "first");
        map.add("k", "second");
        assert_eq!(map.get_first(&"k"), Some(&"first"));
    }

    #[test]
    fn remove_key() {
        let mut map = LinkedMultiValueMap::new();
        map.add("a", 1);
        map.add("b", 2);
        let removed = map.remove(&"a");
        assert_eq!(removed, Some(vec![1]));
        assert!(!map.contains_key(&"a"));
        assert!(map.contains_key(&"b"));
    }

    #[test]
    fn to_single_value_map() {
        let mut map = LinkedMultiValueMap::new();
        map.add("a", 1);
        map.add("a", 2);
        map.add("b", 3);
        let single = map.to_single_value_map();
        assert_eq!(single, vec![("a", 1), ("b", 3)]);
    }

    #[test]
    fn deep_copy_is_independent() {
        let mut map = LinkedMultiValueMap::new();
        map.add("k", "v1");
        let copy = map.deep_copy();
        map.add("k", "v2");
        assert_eq!(map.get_all(&"k"), Some(&["v1", "v2"][..]));
        assert_eq!(copy.get_all(&"k"), Some(&["v1"][..]));
    }

    #[test]
    fn clone_equals_deep_copy() {
        let mut map = LinkedMultiValueMap::new();
        map.add("k", "v1");
        let cloned = map.clone();
        assert_eq!(map.len(), cloned.len());
        assert_eq!(cloned.get_first(&"k"), Some(&"v1"));
    }

    #[test]
    fn clear() {
        let mut map = LinkedMultiValueMap::new();
        map.add("a", 1);
        map.clear();
        assert!(map.is_empty());
    }

    #[test]
    fn debug_format() {
        let mut map = LinkedMultiValueMap::new();
        map.add("k", 42);
        let s = format!("{map:?}");
        assert!(s.contains("k"));
        assert!(s.contains("42"));
    }

    #[test]
    fn default_is_empty() {
        let map: LinkedMultiValueMap<&str, i32> = LinkedMultiValueMap::default();
        assert!(map.is_empty());
    }
}
