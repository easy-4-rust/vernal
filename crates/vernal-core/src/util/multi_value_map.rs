//! 多值 Map。
//!
//! 对标 Spring `org.springframework.util.MultiValueMap`(HTTP headers / form params)。
//!
//! Spring 7 个类都基于 `MultiValueMap<K, List<V>>` 抽象,
//! vernal-core 用一个核心类型 + 适配方法覆盖全部场景。

use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

/// 多值 Map 接口 trait。
///
/// 对标 Spring `MultiValueMap<K, V>` interface。
pub trait MultiValueMapTrait<K, V> {
    /// 获取第一个值(对标 Spring `getFirst`)。
    fn get_first(&self, key: &K) -> Option<&V>;

    /// 添加值(不覆盖,对标 Spring `add`)。
    fn add(&mut self, key: K, value: V);

    /// 设置值(覆盖,对标 Spring `set`)。
    fn set(&mut self, key: K, value: V);

    /// 获取所有值。
    fn get_all(&self, key: &K) -> Option<&[V]>;

    /// 键数量(对标 Spring `Map.size()`)。
    fn len(&self) -> usize;

    /// 是否为空(对标 Spring `Map.isEmpty()`)。
    fn is_empty(&self) -> bool;

    /// 是否包含键(对标 Spring `Map.containsKey()`)。
    fn contains_key(&self, key: &K) -> bool;
}

/// 多值 Map 默认实现。
///
/// 对标 Spring `LinkedMultiValueMap`(基于 `LinkedHashMap` 保持插入顺序)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiValueMap<K: Eq + Hash, V> {
    inner: HashMap<K, Vec<V>>,
}

impl<K: Eq + Hash, V> MultiValueMap<K, V> {
    /// 创建空 Map。
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// 从 `HashMap` 构建。
    #[must_use]
    pub fn from_hashmap(map: HashMap<K, Vec<V>>) -> Self {
        Self { inner: map }
    }

    /// 转换为内部 HashMap(消费 self)。
    #[must_use]
    pub fn into_inner(self) -> HashMap<K, Vec<V>> {
        self.inner
    }

    /// 获取所有键。
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.inner.keys()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// 键数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// 是否包含键。
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.inner.contains_key(key)
    }

    /// 移除键。
    pub fn remove<Q>(&mut self, key: &Q) -> Option<Vec<V>>
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.inner.remove(key)
    }

    /// 清空。
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// 转换为单值 Map(取每个键的第一个值)。
    ///
    /// 对标 Spring `toSingleValueMap()`。
    #[must_use]
    pub fn to_single_value_map(&self) -> HashMap<K, V>
    where
        K: Clone,
        V: Clone,
    {
        self.inner
            .iter()
            .filter_map(|(k, vs)| vs.first().map(|v| (k.clone(), v.clone())))
            .collect()
    }
}

impl<K: Eq + Hash + Clone, V: Clone> MultiValueMapTrait<K, V> for MultiValueMap<K, V> {
    fn get_first(&self, key: &K) -> Option<&V> {
        self.inner.get(key).and_then(|vs| vs.first())
    }

    fn add(&mut self, key: K, value: V) {
        self.inner.entry(key).or_default().push(value);
    }

    fn set(&mut self, key: K, value: V) {
        self.inner.insert(key, vec![value]);
    }

    fn get_all(&self, key: &K) -> Option<&[V]> {
        self.inner.get(key).map(Vec::as_slice)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }
}

impl<K: Eq + Hash, V> Default for MultiValueMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hash, V> Deref for MultiValueMap<K, V> {
    type Target = HashMap<K, Vec<V>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<K: Eq + Hash, V> DerefMut for MultiValueMap<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<K, V> FromIterator<(K, V)> for MultiValueMap<K, V>
where
    K: Eq + Hash,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut map = Self::new();
        for (k, v) in iter {
            map.inner.entry(k).or_default().push(v);
        }
        map
    }
}

/// 不可变包装(对标 Spring `UnmodifiableMultiValueMap`)。
#[derive(Debug, Clone)]
pub struct UnmodifiableMultiValueMap<K: Eq + Hash, V> {
    inner: MultiValueMap<K, V>,
}

impl<K: Eq + Hash, V> UnmodifiableMultiValueMap<K, V> {
    /// 创建不可变包装。
    #[must_use]
    pub fn new(map: MultiValueMap<K, V>) -> Self {
        Self { inner: map }
    }

    /// 获取内部引用。
    #[must_use]
    pub fn as_ref(&self) -> &MultiValueMap<K, V> {
        &self.inner
    }
}

impl<K: Eq + Hash + Clone, V: Clone> MultiValueMapTrait<K, V> for UnmodifiableMultiValueMap<K, V> {
    fn get_first(&self, key: &K) -> Option<&V> {
        self.inner.get_first(key)
    }

    fn add(&mut self, _key: K, _value: V) {
        panic!("UnmodifiableMultiValueMap does not support add");
    }

    fn set(&mut self, _key: K, _value: V) {
        panic!("UnmodifiableMultiValueMap does not support set");
    }

    fn get_all(&self, key: &K) -> Option<&[V]> {
        self.inner.get_all(key)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestMap = MultiValueMap<String, String>;

    fn make_map(pairs: &[(&str, &str)]) -> TestMap {
        let mut m = TestMap::new();
        for (k, v) in pairs {
            m.add((*k).to_string(), (*v).to_string());
        }
        m
    }

    #[test]
    fn add_multiple_values_to_same_key() {
        let m = make_map(&[("h", "v1"), ("h", "v2"), ("h", "v3")]);
        assert_eq!(
            m.get_all(&"h".to_string()),
            Some(&["v1".to_string(), "v2".to_string(), "v3".to_string()][..])
        );
    }

    #[test]
    fn get_first_returns_first_value() {
        let m = make_map(&[("h", "first"), ("h", "second")]);
        assert_eq!(m.get_first(&"h".to_string()), Some(&"first".to_string()));
    }

    #[test]
    fn get_first_returns_none_for_missing() {
        let m: TestMap = MultiValueMap::new();
        assert!(m.get_first(&"missing".to_string()).is_none());
    }

    #[test]
    fn set_overrides_all_values() {
        let mut m = make_map(&[("h", "v1"), ("h", "v2")]);
        m.set("h".to_string(), "new".to_string());
        assert_eq!(m.get_all(&"h".to_string()), Some(&["new".to_string()][..]));
    }

    #[test]
    fn to_single_value_map_takes_first() {
        let m = make_map(&[("a", "1"), ("a", "2"), ("b", "3")]);
        let single = m.to_single_value_map();
        assert_eq!(single.get("a"), Some(&"1".to_string()));
        assert_eq!(single.get("b"), Some(&"3".to_string()));
    }

    #[test]
    fn len_and_is_empty() {
        let mut m: TestMap = MultiValueMap::new();
        assert!(m.is_empty());
        m.add("a".to_string(), "1".to_string());
        assert_eq!(m.len(), 1);
        m.add("b".to_string(), "2".to_string());
        assert_eq!(m.len(), 2);
        m.add("a".to_string(), "3".to_string());
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn contains_key_works() {
        let m = make_map(&[("a", "1")]);
        assert!(m.contains_key(&"a".to_string()));
        assert!(!m.contains_key(&"b".to_string()));
    }

    #[test]
    fn remove_clears_key() {
        let mut m = make_map(&[("a", "1")]);
        let removed = m.remove(&"a".to_string());
        assert_eq!(removed, Some(vec!["1".to_string()]));
        assert!(!m.contains_key(&"a".to_string()));
    }

    #[test]
    fn clear_empties_map() {
        let mut m = make_map(&[("a", "1"), ("b", "2")]);
        m.clear();
        assert!(m.is_empty());
    }

    #[test]
    fn from_iter_collects() {
        let m: TestMap = vec![
            ("a".to_string(), "1".to_string()),
            ("a".to_string(), "2".to_string()),
            ("b".to_string(), "3".to_string()),
        ]
        .into_iter()
        .collect();
        assert_eq!(
            m.get_all(&"a".to_string()),
            Some(&["1".to_string(), "2".to_string()][..])
        );
    }

    #[test]
    fn deref_to_hashmap_works() {
        let m = make_map(&[("a", "1")]);
        assert!(m.contains_key(&"a".to_string()));
    }

    #[test]
    fn into_inner_returns_hashmap() {
        let m = make_map(&[("a", "1")]);
        let inner = m.into_inner();
        assert_eq!(
            inner.get("a").cloned().unwrap_or_default(),
            vec!["1".to_string()]
        );
    }

    #[test]
    fn unmodifiable_panics_on_add() {
        let m = make_map(&[("a", "1")]);
        let mut u = UnmodifiableMultiValueMap::new(m);
        assert_eq!(u.get_first(&"a".to_string()), Some(&"1".to_string()));
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            u.add("b".to_string(), "2".to_string());
        }));
        assert!(result.is_err());
    }

    #[test]
    fn default_is_empty() {
        let m: TestMap = MultiValueMap::default();
        assert!(m.is_empty());
    }

    #[test]
    fn from_hashmap_works() {
        let mut inner = HashMap::new();
        inner.insert("k", vec![1, 2]);
        let map = MultiValueMap::from_hashmap(inner);
        assert_eq!(map.get_first(&"k"), Some(&1));
    }

    #[test]
    fn keys_iterator() {
        let mut map = MultiValueMap::new();
        map.add("a", 1);
        map.add("b", 2);
        let mut keys: Vec<&str> = map.keys().copied().collect();
        keys.sort();
        assert_eq!(keys, vec!["a", "b"]);
    }

    #[test]
    fn trait_len_is_empty_contains() {
        let mut map = MultiValueMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        map.add("k", 1);
        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);
        assert!(map.contains_key(&"k"));
        assert!(!map.contains_key(&"z"));
    }

    #[test]
    fn deref_deref_mut() {
        let mut map = MultiValueMap::new();
        map.add("k", 1);
        // Deref: 直接访问 HashMap 方法
        assert_eq!(map.inner.len(), 1);
        // DerefMut: 直接修改
        map.inner.insert("k2", vec![2]);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn unmodifiable_as_ref() {
        let mut inner = MultiValueMap::new();
        inner.add("k", 1);
        let unmod = UnmodifiableMultiValueMap::new(inner);
        let map_ref = unmod.as_ref();
        assert_eq!(map_ref.get_first(&"k"), Some(&1));
    }

    #[test]
    fn unmodifiable_trait_methods() {
        let mut inner = MultiValueMap::new();
        inner.add("k", 1);
        let unmod = UnmodifiableMultiValueMap::new(inner);
        assert_eq!(unmod.len(), 1);
        assert!(!unmod.is_empty());
        assert!(unmod.contains_key(&"k"));
        assert!(!unmod.contains_key(&"z"));
    }

    #[test]
    #[should_panic(expected = "does not support set")]
    fn unmodifiable_set_panics() {
        let mut inner = MultiValueMap::new();
        let mut unmod = UnmodifiableMultiValueMap::new(inner);
        unmod.set("k", 1);
    }

    #[test]
    fn into_inner_and_default() {
        let map = MultiValueMap::<&str, i32>::default();
        assert!(map.into_inner().is_empty());
    }


    #[test]
    fn deref_and_deref_mut() {
        let mut map = MultiValueMap::new();
        map.add("k", 1);
        // Deref: 直接通过 inner 访问
        assert_eq!(map.inner.len(), 1);
        // DerefMut: 直接修改 inner
        map.inner.insert("k2", vec![2]);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn from_hashmap_and_keys() {
        let mut inner = HashMap::new();
        inner.insert("a", vec![1]);
        inner.insert("b", vec![2]);
        let map = MultiValueMap::from_hashmap(inner);
        let mut keys: Vec<&str> = map.keys().copied().collect();
        keys.sort();
        assert_eq!(keys, vec!["a", "b"]);
    }

    #[test]
    fn trait_len_is_empty_contains_key() {
        let mut map = MultiValueMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        map.add("k", 1);
        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);
        assert!(map.contains_key(&"k"));
        assert!(!map.contains_key(&"z"));
    }

    #[test]
    fn unmodifiable_as_ref_and_trait() {
        let mut inner = MultiValueMap::new();
        inner.add("k", 1);
        let unmod = UnmodifiableMultiValueMap::new(inner);
        let map_ref = unmod.as_ref();
        assert_eq!(map_ref.get_first(&"k"), Some(&1));
        assert_eq!(unmod.len(), 1);
        assert!(!unmod.is_empty());
        assert!(unmod.contains_key(&"k"));
    }

    #[test]
    fn trait_methods_get_all_len_is_empty_contains_key() {
        // 通过 trait 调用所有方法（覆盖 trait impl 而非直接方法）
        let mut m: TestMap = MultiValueMap::new();
        let trait_ref: &dyn MultiValueMapTrait<String, String> = &m;
        assert_eq!(trait_ref.len(), 0);
        assert!(trait_ref.is_empty());
        assert!(!trait_ref.contains_key(&"missing".to_string()));

        m.add("h".to_string(), "v1".to_string());
        m.add("h".to_string(), "v2".to_string());
        m.add("o".to_string(), "other".to_string());

        let trait_ref: &dyn MultiValueMapTrait<String, String> = &m;
        assert_eq!(trait_ref.len(), 2);
        assert!(!trait_ref.is_empty());
        assert!(trait_ref.contains_key(&"h".to_string()));
        assert!(!trait_ref.contains_key(&"missing".to_string()));
        // get_all 通过 trait 返回切片的引用
        let all = trait_ref.get_all(&"h".to_string()).unwrap();
        assert_eq!(all, &["v1".to_string(), "v2".to_string()][..]);
    }

    #[test]
    fn deref_mut_allows_hashmap_entry_api() {
        // 对标 Spring `LinkedMultiValueMap` 基于 LinkedHashMap 的 mut 访问
        let mut map: MultiValueMap<&str, i32> = MultiValueMap::new();
        map.add("counter", 1);
        // 通过 DerefMut 直接修改内部 HashMap
        {
            let inner: &mut HashMap<&str, Vec<i32>> = &mut *map;
            inner.entry("counter").or_default().push(2);
        }
        assert_eq!(map.get_all(&"counter"), Some(&[1, 2][..]));
    }

    #[test]
    fn deref_provides_hashmap_methods() {
        let mut map: MultiValueMap<&str, i32> = MultiValueMap::new();
        map.add("a", 1);
        map.add("b", 2);
        // Deref 暴露 HashMap 的 capability 方法
        let inner: &HashMap<&str, Vec<i32>> = &*map;
        assert_eq!(inner.len(), 2);
        assert!(inner.contains_key(&"a"));
        assert!(!inner.contains_key(&"c"));
    }

    #[test]
    fn unmodifiable_trait_get_all_returns_inner_slice() {
        // 对标 Spring `UnmodifiableMultiValueMap.getCollection()` 委托给内部
        let mut inner: MultiValueMap<&str, i32> = MultiValueMap::new();
        inner.add("k", 10);
        inner.add("k", 20);
        let unmod = UnmodifiableMultiValueMap::new(inner);
        // 通过 trait 调用 get_all
        let trait_ref: &dyn MultiValueMapTrait<&str, i32> = &unmod;
        let all = trait_ref.get_all(&"k").unwrap();
        assert_eq!(all, &[10, 20][..]);
    }

}