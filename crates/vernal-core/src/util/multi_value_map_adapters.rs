//! MultiValueMap 适配器家族。
//!
//! 对标 Spring `org.springframework.util` 中的适配器。

use std::collections::HashMap;
use std::hash::Hash;
use super::multi_value_map::MultiValueMapTrait;

/// 包装任意 `Map<K, Vec<V>>` 实现 `MultiValueMapTrait`。对标 Spring MultiValueMapAdapter。
pub struct MultiValueMapAdapter<K: Eq + Hash, V> {
    target: HashMap<K, Vec<V>>,
}

impl<K: Eq + Hash, V> MultiValueMapAdapter<K, V> {
    #[must_use]
    pub fn new(target: HashMap<K, Vec<V>>) -> Self { Self { target } }
    #[must_use]
    pub fn len(&self) -> usize { self.target.len() }
    #[must_use]
    pub fn is_empty(&self) -> bool { self.target.is_empty() }
    #[must_use]
    pub fn to_single_value_map(&self) -> HashMap<K, V> where K: Clone, V: Clone {
        self.target.iter().filter_map(|(k, vs)| vs.first().map(|v| (k.clone(), v.clone()))).collect()
    }
    #[must_use]
    pub fn as_inner(&self) -> &HashMap<K, Vec<V>> { &self.target }
    pub fn as_inner_mut(&mut self) -> &mut HashMap<K, Vec<V>> { &mut self.target }
    #[must_use]
    pub fn into_inner(self) -> HashMap<K, Vec<V>> { self.target }
}

impl<K: Eq + Hash, V> MultiValueMapTrait<K, V> for MultiValueMapAdapter<K, V> {
    fn get_first(&self, key: &K) -> Option<&V> { self.target.get(key).and_then(|vs| vs.first()) }
    fn add(&mut self, key: K, value: V) { self.target.entry(key).or_default().push(value); }
    fn set(&mut self, key: K, value: V) { self.target.insert(key, vec![value]); }
    fn get_all(&self, key: &K) -> Option<&[V]> { self.target.get(key).map(Vec::as_slice) }
    fn len(&self) -> usize { self.target.len() }
    fn is_empty(&self) -> bool { self.target.is_empty() }
    fn contains_key(&self, key: &K) -> bool { self.target.contains_key(key) }
}

impl<K: Eq + Hash, V> Default for MultiValueMapAdapter<K, V> {
    fn default() -> Self { Self::new(HashMap::new()) }
}

/// 多值 Map → 单值 Map 实时视图。对标 Spring MultiToSingleValueMapAdapter。
pub struct MultiToSingleValueMapAdapter<'a, K: Eq + Hash, V, M: MultiValueMapTrait<K, V>> {
    delegate: &'a M,
    _k: std::marker::PhantomData<K>,
    _v: std::marker::PhantomData<V>,
}

impl<'a, K: Eq + Hash, V, M: MultiValueMapTrait<K, V>> MultiToSingleValueMapAdapter<'a, K, V, M> {
    #[must_use]
    pub fn new(delegate: &'a M) -> Self {
        Self { delegate, _k: std::marker::PhantomData, _v: std::marker::PhantomData }
    }
    #[must_use]
    pub fn get(&self, key: &K) -> Option<&V> { self.delegate.get_first(key) }
    #[must_use]
    pub fn len(&self) -> usize { self.delegate.len() }
    #[must_use]
    pub fn is_empty(&self) -> bool { self.delegate.is_empty() }
    #[must_use]
    pub fn contains_key(&self, key: &K) -> bool { self.delegate.contains_key(key) }
}

/// 单值 Map → 多值 Map 包装。对标 Spring SingleToMultiValueMapAdapter。
pub struct SingleToMultiValueMapAdapter<K: Eq + Hash, V> {
    target: HashMap<K, V>,
}

impl<K: Eq + Hash, V> SingleToMultiValueMapAdapter<K, V> {
    #[must_use]
    pub fn new(target: HashMap<K, V>) -> Self { Self { target } }
    #[must_use]
    pub fn len(&self) -> usize { self.target.len() }
    #[must_use]
    pub fn is_empty(&self) -> bool { self.target.is_empty() }
    #[must_use]
    pub fn into_single_value_map(self) -> HashMap<K, V> { self.target }
}

impl<K: Eq + Hash + Clone, V: Clone> MultiValueMapTrait<K, V> for SingleToMultiValueMapAdapter<K, V> {
    fn get_first(&self, key: &K) -> Option<&V> { self.target.get(key) }
    fn add(&mut self, key: K, value: V) { self.target.entry(key).or_insert(value); }
    fn set(&mut self, key: K, value: V) { self.target.insert(key, value); }
    fn get_all(&self, _key: &K) -> Option<&[V]> { None }
    fn len(&self) -> usize { self.target.len() }
    fn is_empty(&self) -> bool { self.target.is_empty() }
    fn contains_key(&self, key: &K) -> bool { self.target.contains_key(key) }
}

/// 多值 Map 的 Stream 收集器。对标 Spring MultiValueMapCollector。
pub struct MultiValueMapCollector<K: Eq + Hash, V> {
    map: HashMap<K, Vec<V>>,
}

impl<K: Eq + Hash, V> MultiValueMapCollector<K, V> {
    #[must_use]
    pub fn into_map(self) -> HashMap<K, Vec<V>> { self.map }
}

impl<K: Eq + Hash, V> Default for MultiValueMapCollector<K, V> {
    fn default() -> Self { Self { map: HashMap::new() } }
}

impl<K: Eq + Hash, V> Extend<(K, V)> for MultiValueMapCollector<K, V> {
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        for (k, v) in iter { self.map.entry(k).or_default().push(v); }
    }
}

impl<K: Eq + Hash, V> FromIterator<(K, V)> for MultiValueMapCollector<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut c = Self::default(); c.extend(iter); c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_basic() {
        let mut inner = HashMap::new();
        inner.insert("k", vec![1, 2]);
        let a = MultiValueMapAdapter::new(inner);
        assert_eq!(a.get_first(&"k"), Some(&1));
        assert_eq!(a.get_all(&"k"), Some(&[1, 2][..]));
        assert_eq!(a.len(), 1);
        assert!(!a.is_empty());
        assert!(a.contains_key(&"k"));
        assert!(!a.contains_key(&"z"));
    }

    #[test]
    fn adapter_add_set() {
        let mut a = MultiValueMapAdapter::default();
        a.add("k", 1);
        a.add("k", 2);
        assert_eq!(a.get_all(&"k"), Some(&[1, 2][..]));
        a.set("k", 99);
        assert_eq!(a.get_all(&"k"), Some(&[99][..]));
    }

    #[test]
    fn adapter_as_inner() {
        let mut inner = HashMap::new();
        inner.insert("k", vec![1]);
        let a = MultiValueMapAdapter::new(inner);
        assert_eq!(a.as_inner().len(), 1);
    }

    #[test]
    fn adapter_as_inner_mut() {
        let mut a = MultiValueMapAdapter::default();
        a.as_inner_mut().insert("k", vec![42]);
        assert_eq!(a.get_first(&"k"), Some(&42));
    }

    #[test]
    fn adapter_to_single_value_map() {
        let mut inner = HashMap::new();
        inner.insert("a", vec![1, 2]);
        inner.insert("b", vec![3]);
        let a = MultiValueMapAdapter::new(inner);
        let single = a.to_single_value_map();
        assert_eq!(single.get("a"), Some(&1));
        assert_eq!(single.get("b"), Some(&3));
    }

    #[test]
    fn adapter_into_inner() {
        let mut a = MultiValueMapAdapter::default();
        a.add("k", 42);
        let inner = a.into_inner();
        assert_eq!(inner.get("k"), Some(&vec![42]));
    }

    #[test]
    fn view_get_len_is_empty_contains() {
        let mut inner = HashMap::new();
        inner.insert("k", vec![10]);
        let a = MultiValueMapAdapter::new(inner);
        let v = MultiToSingleValueMapAdapter::new(&a);
        assert_eq!(v.get(&"k"), Some(&10));
        assert_eq!(v.len(), 1);
        assert!(!v.is_empty());
        assert!(v.contains_key(&"k"));
        assert!(!v.contains_key(&"z"));
    }

    #[test]
    fn view_empty() {
        let a = MultiValueMapAdapter::<&str, i32>::default();
        let v = MultiToSingleValueMapAdapter::new(&a);
        assert!(v.is_empty());
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn single_to_multi_basic() {
        let mut single = HashMap::new();
        single.insert("a", 1);
        single.insert("b", 2);
        let a = SingleToMultiValueMapAdapter::new(single);
        assert_eq!(a.get_first(&"a"), Some(&1));
        assert_eq!(a.len(), 2);
        assert!(!a.is_empty());
        assert!(a.contains_key(&"a"));
        assert!(!a.contains_key(&"z"));
        assert_eq!(a.get_all(&"a"), None);
    }

    #[test]
    fn single_to_multi_add_set() {
        let mut a = SingleToMultiValueMapAdapter::new(HashMap::new());
        a.add("k", 1);
        assert_eq!(a.get_first(&"k"), Some(&1));
        a.set("k", 99);
        assert_eq!(a.get_first(&"k"), Some(&99));
    }

    #[test]
    fn single_to_multi_into() {
        let mut single = HashMap::new();
        single.insert("x", 10);
        let a = SingleToMultiValueMapAdapter::new(single);
        let map = a.into_single_value_map();
        assert_eq!(map.get("x"), Some(&10));
    }

    #[test]
    fn collector_from_iter() {
        let pairs = vec![("a", 1), ("a", 2), ("b", 3)];
        let c: MultiValueMapCollector<_, _> = pairs.into_iter().collect();
        let map = c.into_map();
        assert_eq!(map.get("a"), Some(&vec![1, 2]));
        assert_eq!(map.get("b"), Some(&vec![3]));
    }

    #[test]
    fn collector_empty() {
        let c: MultiValueMapCollector<&str, i32> = std::iter::empty::<(&str, i32)>().collect();
        assert!(c.into_map().is_empty());
    }
}
