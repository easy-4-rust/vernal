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
}
